# No Std Framework Arduino - TODO

- [ ] Support missing behaviours.
    - [x] `TickerBehaviour`
    - [ ] `ParallelBehaviour`
- [x] Support message passing to the parent.
- [ ] Support missing actions on the context.
    - [ ] Adding behaviours.
    - [ ] Removing behaviours.
    - [x] Deleting the agent.
    - [x] Stopping the container.
    - [x] Blocking behaviours.
- [ ] Fix leaking memory when passing the content of the behaviour unique pointer to rust.
