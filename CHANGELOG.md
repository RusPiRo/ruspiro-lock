# Changelog

## :peach: v0.4.0

This version provides a major refactoring to introduce commonly used names for the different kinds of locks. `Datalock` becomes `Mutex` and `DataRWlock` becomes `RWLock`. With a feature gate also `async` versions of those locks are introduced.

- ### :bulb: Features

  - Provide async mutex and semaphore versions

- ### :wrench: Maintenance

  - Rename `DataLock` to `Mutex`
  - Rename `DataRWLock` to `RWLock`
  - Introducing the enhances travis-ci pipeline to build and publish this crate

## :carrot: v0.3.3

- ### :bulb: Features

  - Introduce a ``DataRWLock`` that enhances the ``DataLock`` in a way that in addition to mutual exclusive access a
    read-only access is also possible.

- ### :wrench: Maintenance

  - The ``Semaphore`` does now support counter of type ``u32`` instead of ``u16``.
  - Flag some of the lock functions to allow the compiler to inline them as port of the optimization to increase performance
  - Use ``cargo-make`` for convinient and reliable cross platform build execution to reduce maintenance efforts for local and CI builds.

- ### :detective: Fixes

  Fix issues with ``Semaphore``, ``DataLock`` and ``Spinlock`` that sometimes does not released a protected resource due to missing data memory and data syncronisation barriers not used as pointed out in this document: http://infocenter.arm.com/help/topic/com.arm.doc.dht0008a/DHT0008A_arm_synchronization_primitives.pdf

## :carrot: v0.3.2

- ### :wrench: Maintenance

  - Remove the dependency to the interrupts. Locking will no longer disable/re-enable interrupt handling globally. Usage of blocking locks need to avoided inside interrupt handler to mitigate the risk of deadlocks.

## :carrot: v0.3.1

- ### :bulb: Features

  Introduce a blocking ``lock`` function for the ``DataLock``.

- ### :wrench: Maintenance

  - Apply code quality improvements based on ``clippy``
  - remove ``ruspiro_pi3`` feature gate as it is not needed/used

- ### :book: Documentation

  - Adjust documentation to reflect the current state of developments
