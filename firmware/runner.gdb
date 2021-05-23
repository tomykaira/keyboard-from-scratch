# break DefaultHandler
# break UserHardFault
# break rust_begin_unwind
monitor reset halt
set mem inaccessible-by-default off
monitor arm semihosting enable
load
compare-sections
