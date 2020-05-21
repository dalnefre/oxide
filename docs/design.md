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

## Micro-Kernel

Ultimately all work is done by a _processing unit_ (core).
The micro-kernel multiplexes cores across the work-in-process.
Some cores may be special-purpose, and accept limited types of work.

A _configuration_ is a collection of _actors_ and pending message _events_.
The _actors_ capture the state of the system.
The _events_ capture the work-in-process.
The micro-kernel is optimized around the delivery of asynchronous _events_.
Each event delivers a unique _message_ to a target _actor_,
invoking the current _behavior_ of that actor.

The _behavior_ of an actor is a resource-bounded computation
producing a set of effects, which are limited to:

  * creating new actors
  * sending new messages
  * updating the actor's local state
  * updating the actor's behavior for subsequent messages
  * signaling a failure

The actor's response to a message is an atomic transaction.
Either all computed effects become visible at once (commit),
or no effects apply and a failure is signaled (roll-back).

Resources (primarily processing time and storage space) are controlled by a _sponsor_.
Each message _event_ has a sponsor
from whom the _actor_ obtains resources and to whom it reports results.

Configurations form a parent-child tree structure,
with the parent responsible for scheduling among its children.
Sponsors may be shared across configurations,
and/or created to manage fine-grained computations.
Sponsors also form a parent-child tree structure,
where a parent may sub-divide its resources among its children.
