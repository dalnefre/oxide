# Oxide Design

[Oxide](READMD.md) has a very small asynchronous message-passing micro-kernel on which all other facilities are built.
The three most important design requirements are:
  * Low latency
  * High throughput
  * Provable safety

**Low latency** is achieved by ...
  * common address space with guaranteed memory-safety
  * very fine-grained cooperative-multitasking
  * very fast context switching (little state to save)
  * bounded buffers to limit work-in-process

**High throughput** is achieved by ...
  * avoid copying data
  * share immutable values
  * minimize representation conversions
  * "single-piece" lean workflow

**Provable safety** is achieved by ...
  * memory-safe computational model (no mutable shared-state)
  * fine-grained capability-security for all access
  * scalable abstraction/interface boundaries
  * pervasive failure monitoring
  * modular live-update/restart
  * transactional persistence
  * sound mathematical/logical theory
