# Library checklist / TODOs

## Core functionality
- [x] DCP initialization
- [ ] Check for capabilities on init
- [x] Using multiple channels
- [ ] Channel/Scheduler singletons
- [x] Raw control packet API
- [x] Packet builder API
- [x] Pollable operation handles
- [x] Execute packets
- [ ] Execute multiple packets
    - [x] Contiguous slice of packets
    - [ ] Using the `next` field of the packet
    - [ ] API for building packet chains
- [ ] Making sure it works

## Optimizations
- [ ] Align input/output buffers
- [ ] Smart allocation of context switching buffers
- [ ] Payload builder API

## Features
- [ ] Traits for cypto/digest operations
- [x] Multi-channel scheduler
- [ ] High-priority and reserved channels
- [ ] Manage AES keys (OTP, write-only memory)

## Library
- [x] Prelude with essential structs and traits
- [x] Select SoC through features
- [ ] Find a way to run tests
- [ ] Useful documentation (possibly serious, so not written by me)

## Improvements
- [x] Reusable builders
- [x] Set buffer addresses in the operation and not the builder (yaay, more type-specific impls)
- [ ] Require setting buffer positions before freezing task
- [ ] `alloc` feature for packet chains and dynamic queues
