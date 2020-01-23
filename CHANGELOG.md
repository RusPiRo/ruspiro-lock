# Changelog
## :carrot: v0.3.2
  - ### :wrench: Maintenance
    - Remove the dependency to the interrupts. Locking will no longer disable/re-enable interrupt handling globally.
    Usage of blocking locks need to avoided inside interrupt handler to mitigate the risk of deadlocks.
    
## :carrot: v0.3.1
  - ### :bulb: Features
    Introduce a blocking ``lock`` function for the ``DataLock``.
  - ### :wrench: Maintenance
    - Apply code quality improvements based on ``clippy``
    - remove ``ruspiro_pi3`` feature gate as it is not needed/used
  - ### :book: Documentation
    - Adjust documentation to reflect the current state of developments
