# Oxide Design

[Oxide](../README.md) has a very small asynchronous message-passing micro-kernel on which all other facilities are built.
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

The primary hardware platform for Oxide is a many-core x86-64 machine with a large main memory.
We generally prefer to trade memory for higher performance, in terms of both latency and throughput.
In addition, pre-allocation and configuration of fixed-size resource pools reduces variability in response times.

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

### Bootstrap

On start-up, a single core executes the boot process.
This starts a single root actor configuration.
The root configuration creates child configurations
representing sub-systems defined by a load-out image.
Additional cores are activated
to support concurrent dispatch of message-events
in multiple configurations.
The root configuration becomes the top-level system monitor,
handling failures, restarts, and reconfigurations
within the configuration tree.

### Event Deques

Each _configuration_ maintains a set of _actors_ and a set of pending message _events_.
All of the work to be done within a configuration is represented by the pending-event set.
Adding events (sending messages) and dispatching events (invoking actor behaviors)
must be optimized for the runtime to exhibit good performance.

We propose a deque,
implemented by a fixed-size ring-buffer of cache-line sized event slots,
as the central message-event dispatch data-structure.
Events are dispatched from the head of the deque,
where they can be referenced by the target actor's behavior
without making a copy, since they are read-only.
While executing a behavior,
new events may be speculatively added
to the tail of the deque.
If the behavior signals failure,
the tail-pointer is reset to discard the uncommitted events.
Interrupt-handlers may add events before the head of the deque,
ensuring that they are dispatched with priority
and avoiding interference with uncommitted events.
Sponsors will enforce limits that prevent overflow conditions.
