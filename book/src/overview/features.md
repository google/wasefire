# Features

Features are implemented if their checkbox is checked.

If a feature is not implemented, it doesn't mean it will get implemented. It means this is a feature
that could be implemented if there is a user need.

If a feature is not listed, it doesn't mean it won't get implemented. We may just not be aware of
it, and a user need could justify an implementation.

## Supported boards

A board is supported if it has a Runner.

- [x] Linux (for testing without hardware)
- [x] nRF52840
- [x] OpenTitan

## Supported applet languages

An applet language is supported if it has a Prelude.

- [x] Rust
- [ ] C
- [ ] AssemblyScript

Note that when running multiple applets concurrently on the same platform, those applets don't need
to be written in the same language to inter-operate.

## Developer experience

### For applets

- [x] Development doesn't require hardware (using the Linux board).
- [x] Testing facilities (using `abort()` and `exit()`).
- [ ] Fuzzing facilities.
- [ ] Debugging facilities.

### For runners

- [x] Testing facilities (using the set of example applets suffixed with `_test`).

## Reproducible builds

- [ ] Hermetic development environment for applets.
- [ ] Hermetic development environment for platforms.

## Secure platform upgrades

- [x] The platform can be upgraded.
- [ ] The platform can be downgraded to the extent permitted by the User-configured rollback policy.
- [ ] Platform upgrades are digitally signed and verified.

## Applet sandboxing

- [x] Applets can't tamper with the platform.
- [ ] Applets can't tamper with other applets (this is only missing preemptive concurrency).

## Applet capabilities

- [x] Applets declare their permissions[^applet-perms].
- [ ] Applets declare their capabilities (more dynamic concept of permission).
- [ ] Applets metadata (or manifest) is signed.

[^applet-perms]: This is currently done using the set of imported functions in the WebAssembly
    module of the applet. This will most probably use the manifest mechanism in the future.

## Platform side-channel attack testing and resistance

- [x] Crypto hardware accelerators are leveraged when available (board specific).
- [x] Software crypto primitives are provided as fallback.
- [ ] Both of those implementations are side-channel attack resilient.

## Applet portability

- [x] Applets are portable at binary level (comes from WebAssembly and APIs).

## Applet multiplexing

- [ ] Multiple applets may be installed at the same time.
- [ ] Multiple applets may run simultaneously.
- [ ] Applets can be installed without running.
- [ ] Applets define in their metadata their running condition (e.g. at boot, at USB, at idle, etc).

For now, at most one applet can be installed. It immediately starts running after it has been
installed and when the platform boots.

## Applet management

- [ ] Applets are identified by a stable id, a version, and a digital signature (verified by the
      runtime).
- [x] Applets may be installed if not already present (within the limit of one applet).
- [ ] Applets may be uninstalled in which case all owned resources are deleted.
- [ ] Applets may be upgraded (preserving resources) but not downgraded (probably modulo rollback
      policy).
- [ ] Installed applets can be listed.

Applets can be uninstalled and upgraded, but the resources are untouched because there is no mapping
of resources to applets yet (there is no stable identifier).

## Certification

- [ ] The runtime can run on certified hardware (FIPS-140-3 and CC).
- [ ] TBD: The runtime might sustain being part of the security target for certification.

## Low power

- [x] If the runtime is only waiting on external hardware events, the CPU is suspended.
