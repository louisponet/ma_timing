#include <cstdint>
namespace ma_timing {

struct Timer {
	uint8_t data[192];
};
extern "C" {
	void create_timer(const char* name, Timer* timer);
	void start(Timer* timer);
	void stop(Timer* timer);
	void latency(Timer* timer, uint64_t rdtscp_timestamp);
}
}
