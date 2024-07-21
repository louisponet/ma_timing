// Will be a rolling buffer to be used with data that gets streamed in.
// Before the buffer is filled fully once, the data is garbage, i.e.
// be careful what you iterate over
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    data:   Vec<T>,
    mask:   usize,
    // Keep track of which pos is current begin
    filled: bool,
    // Which box to fill NEXT, i.e. id of the first element in buffer
    pos:    usize,
}

impl<T: Default + Clone> CircularBuffer<T> {
    pub fn new(size: usize) -> Self {
        let realsize = size.next_power_of_two();
        Self { data: vec![T::default(); size], mask: realsize - 1, filled: false, pos: 0 }
    }
}
impl<T> CircularBuffer<T> {
    pub fn push(&mut self, v: T) {
        unsafe { *self.data.get_unchecked_mut(self.pos) = v };
        self.pos = (self.pos + 1) & self.mask;
        self.filled |= self.pos == 0;
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self)
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.filled {
            self.data.len()
        } else {
            self.pos
        }
    }

    #[inline]
    pub fn last(&self) -> &T {
        let pos = if self.pos > 1 { self.pos - 1 } else { self.mask };
        unsafe { self.data.get_unchecked(pos) }
    }

    fn first_pos(&self) -> usize {
        if self.filled {
            self.pos
        } else {
            0
        }
    }
}

pub struct Iter<'a, T> {
    // This guy goes around
    pos:    usize,
    count:  usize,
    // How many
    buffer: &'a CircularBuffer<T>,
}

impl<'a, T> Iter<'a, T> {
    fn new(buffer: &'a CircularBuffer<T>) -> Self {
        Self { pos: buffer.first_pos(), count: 0, buffer }
    }
}

pub struct IterMut<'a, T> {
    pos:    usize,
    count:  usize,
    // How many
    buffer: &'a mut CircularBuffer<T>,
}

impl<'a, T> IterMut<'a, T> {
    fn new(buffer: &'a mut CircularBuffer<T>) -> Self {
        Self { pos: buffer.first_pos(), count: 0, buffer }
    }
}

impl<'a, T: Copy> IntoIterator for &'a CircularBuffer<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Copy> IntoIterator for &'a mut CircularBuffer<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.buffer.len() {
            return None;
        }
        let out = unsafe { self.buffer.data.get_unchecked(self.pos & self.buffer.mask) };
        self.pos = (self.pos + 1) & self.buffer.mask;
        self.count += 1;
        Some(out)
    }
}
impl<'a, T> Iterator for IterMut<'a, T> where T: 'a
{
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count == self.buffer.len() {
            return None;
        }

        unsafe {
            let elem = self.buffer.data.get_unchecked_mut((self.pos - 1) & self.buffer.mask);
            self.pos = (self.pos + 1) & self.buffer.mask;
            self.count += 1;
            Some(&mut *(elem as *mut T))
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test() {
        let mut buf = CircularBuffer::new(32);
        let mut tot = 0;
        for i in 0..30 {
            buf.push(i);
            tot += i;
        }

        assert_eq!(tot, buf.iter().sum::<i32>());

        let mut buf = CircularBuffer::new(32);
        let mut tot = 0;
        for i in 1..36 {
            buf.push(i);
            if i > 3 {
                tot += i;
            }
        }
        assert_eq!(tot, buf.iter().sum::<i32>());
    }
}
