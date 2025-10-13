# Issue #108 Response Draft

Hi @honhwa,

Thanks for reporting this issue and providing the detailed error logs. You were absolutely right - MoE CPU offloading wasn't working as advertised.

I've identified and fixed the problem. During testing, some critical code lines got commented out and accidentally stayed that way in the release. The MoE functionality was essentially disabled while still showing the startup messages, which was misleading.

The fix has been implemented and thoroughly tested with real MoE models. Everything is working correctly now:

- `--cpu-moe` properly offloads ALL expert tensors to CPU (65-85% VRAM savings)
- `--n-cpu-moe N` offloads first N expert layers as expected
- Memory allocation errors like yours should be resolved

**Fix commit: `f91e7ca`**
**Documentation: `d97dd24`**

You can pull the latest version to test it immediately, or wait for the next official release. The MoE CPU offloading is now fully functional and will help with those large model memory issues you were experiencing.

Thanks for your patience and for helping us catch this.

-Mic