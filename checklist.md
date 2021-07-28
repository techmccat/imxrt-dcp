# Library checklist / TODOs

## Core functionality
- [x] DCP initialization
- [ ] Check for capabilities on init
- [ ] Using multiple channels
- [ ] Channel singletons
- [x] Raw control packet API
- [x] Packet builder API
- [x] Pollable operation handles
- [ ] Execute packets
- [ ] Execute multiple packets
- [ ] Making sure it works

## Optimizations
- [ ] Align input/output buffers
- [ ] Smart allocation of context switching buffers
- [ ] Payload builder API

## Features
- [ ] Traits for cypto/digest operations
- [ ] Multi-channel scheduler
- [ ] Manage AES keys (OTP, write-only memory)

## Library
- [x] Prelude with essential structs and traits
- [x] Select SoC through features
- [ ] Find a way to run tests
- [ ] Useful documentation (possibly serious, so not written by me)

## Improvements
- [x] Reusable builders
- [x] Set buffer addresses in the operation and not the builder (yaay, more type-specific impls)
- [ ] HAL feature to take CCM handle instead of registers
- [ ] Require setting buffer positions before freezing task
