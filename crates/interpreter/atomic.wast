;; atomic operations

(module
  (memory 1 1 shared)

  (func (export "init") (param $value i64) (i64.store (i32.const 0) (local.get $value)))

  (func (export "i32.atomic.load") (param $addr i32) (result i32) (i32.atomic.load (local.get $addr)))
  (func (export "i64.atomic.load") (param $addr i32) (result i64) (i64.atomic.load (local.get $addr)))
  (func (export "i32.atomic.load8_u") (param $addr i32) (result i32) (i32.atomic.load8_u (local.get $addr)))
  (func (export "i32.atomic.load16_u") (param $addr i32) (result i32) (i32.atomic.load16_u (local.get $addr)))
  (func (export "i64.atomic.load8_u") (param $addr i32) (result i64) (i64.atomic.load8_u (local.get $addr)))
  (func (export "i64.atomic.load16_u") (param $addr i32) (result i64) (i64.atomic.load16_u (local.get $addr)))
  (func (export "i64.atomic.load32_u") (param $addr i32) (result i64) (i64.atomic.load32_u (local.get $addr)))

  (func (export "i32.atomic.store") (param $addr i32) (param $value i32) (i32.atomic.store (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.store") (param $addr i32) (param $value i64) (i64.atomic.store (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.store8") (param $addr i32) (param $value i32) (i32.atomic.store8 (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.store16") (param $addr i32) (param $value i32) (i32.atomic.store16 (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.store8") (param $addr i32) (param $value i64) (i64.atomic.store8 (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.store16") (param $addr i32) (param $value i64) (i64.atomic.store16 (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.store32") (param $addr i32) (param $value i64) (i64.atomic.store32 (local.get $addr) (local.get $value)))

  (func (export "i32.atomic.rmw.add") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw.add (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw.add") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw.add (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw8.add_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw8.add_u (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw16.add_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw16.add_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw8.add_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw8.add_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw16.add_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw16.add_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw32.add_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw32.add_u (local.get $addr) (local.get $value)))

  (func (export "i32.atomic.rmw.sub") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw.sub (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw.sub") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw.sub (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw8.sub_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw8.sub_u (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw16.sub_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw16.sub_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw8.sub_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw8.sub_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw16.sub_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw16.sub_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw32.sub_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw32.sub_u (local.get $addr) (local.get $value)))

  (func (export "i32.atomic.rmw.and") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw.and (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw.and") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw.and (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw8.and_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw8.and_u (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw16.and_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw16.and_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw8.and_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw8.and_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw16.and_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw16.and_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw32.and_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw32.and_u (local.get $addr) (local.get $value)))

  (func (export "i32.atomic.rmw.or") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw.or (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw.or") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw.or (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw8.or_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw8.or_u (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw16.or_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw16.or_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw8.or_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw8.or_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw16.or_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw16.or_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw32.or_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw32.or_u (local.get $addr) (local.get $value)))

  (func (export "i32.atomic.rmw.xor") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw.xor (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw.xor") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw.xor (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw8.xor_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw8.xor_u (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw16.xor_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw16.xor_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw8.xor_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw8.xor_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw16.xor_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw16.xor_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw32.xor_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw32.xor_u (local.get $addr) (local.get $value)))

  (func (export "i32.atomic.rmw.xchg") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw.xchg (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw.xchg") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw.xchg (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw8.xchg_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw8.xchg_u (local.get $addr) (local.get $value)))
  (func (export "i32.atomic.rmw16.xchg_u") (param $addr i32) (param $value i32) (result i32) (i32.atomic.rmw16.xchg_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw8.xchg_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw8.xchg_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw16.xchg_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw16.xchg_u (local.get $addr) (local.get $value)))
  (func (export "i64.atomic.rmw32.xchg_u") (param $addr i32) (param $value i64) (result i64) (i64.atomic.rmw32.xchg_u (local.get $addr) (local.get $value)))

  (func (export "i32.atomic.rmw.cmpxchg") (param $addr i32) (param $expected i32) (param $value i32) (result i32) (i32.atomic.rmw.cmpxchg (local.get $addr) (local.get $expected) (local.get $value)))
  (func (export "i64.atomic.rmw.cmpxchg") (param $addr i32) (param $expected i64)  (param $value i64) (result i64) (i64.atomic.rmw.cmpxchg (local.get $addr) (local.get $expected) (local.get $value)))
  (func (export "i32.atomic.rmw8.cmpxchg_u") (param $addr i32) (param $expected i32)  (param $value i32) (result i32) (i32.atomic.rmw8.cmpxchg_u (local.get $addr) (local.get $expected) (local.get $value)))
  (func (export "i32.atomic.rmw16.cmpxchg_u") (param $addr i32) (param $expected i32)  (param $value i32) (result i32) (i32.atomic.rmw16.cmpxchg_u (local.get $addr) (local.get $expected) (local.get $value)))
  (func (export "i64.atomic.rmw8.cmpxchg_u") (param $addr i32) (param $expected i64)  (param $value i64) (result i64) (i64.atomic.rmw8.cmpxchg_u (local.get $addr) (local.get $expected) (local.get $value)))
  (func (export "i64.atomic.rmw16.cmpxchg_u") (param $addr i32) (param $expected i64)  (param $value i64) (result i64) (i64.atomic.rmw16.cmpxchg_u (local.get $addr) (local.get $expected) (local.get $value)))
  (func (export "i64.atomic.rmw32.cmpxchg_u") (param $addr i32) (param $expected i64)  (param $value i64) (result i64) (i64.atomic.rmw32.cmpxchg_u (local.get $addr) (local.get $expected) (local.get $value)))

)

;; *.atomic.load*

(invoke "init" (i64.const 0x0706050403020100))

(assert_trap (invoke "i32.atomic.load" (i32.const 0)) "not implemented")
(assert_trap (invoke "i32.atomic.load" (i32.const 4)) "not implemented")

(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(assert_trap (invoke "i32.atomic.load8_u" (i32.const 0)) "not implemented")
(assert_trap (invoke "i32.atomic.load8_u" (i32.const 5)) "not implemented")

(assert_trap (invoke "i32.atomic.load16_u" (i32.const 0)) "not implemented")
(assert_trap (invoke "i32.atomic.load16_u" (i32.const 6)) "not implemented")

(assert_trap (invoke "i64.atomic.load8_u" (i32.const 0)) "not implemented")
(assert_trap (invoke "i64.atomic.load8_u" (i32.const 5)) "not implemented")

(assert_trap (invoke "i64.atomic.load16_u" (i32.const 0)) "not implemented")
(assert_trap (invoke "i64.atomic.load16_u" (i32.const 6)) "not implemented")

(assert_trap (invoke "i64.atomic.load32_u" (i32.const 0)) "not implemented")
(assert_trap (invoke "i64.atomic.load32_u" (i32.const 4)) "not implemented")

;; *.atomic.store*

(invoke "init" (i64.const 0x0000000000000000))

(assert_trap (invoke "i32.atomic.store" (i32.const 0) (i32.const 0xffeeddcc))  "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(assert_trap (invoke "i64.atomic.store" (i32.const 0) (i64.const 0x0123456789abcdef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(assert_trap (invoke "i32.atomic.store8" (i32.const 1) (i32.const 0x42)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(assert_trap (invoke "i32.atomic.store16" (i32.const 4) (i32.const 0x8844)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(assert_trap (invoke "i64.atomic.store8" (i32.const 1) (i64.const 0x99)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(assert_trap (invoke "i64.atomic.store16" (i32.const 4) (i64.const 0xcafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(assert_trap (invoke "i64.atomic.store32" (i32.const 4) (i64.const 0xdeadbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.add

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.add" (i32.const 0) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.add" (i32.const 0) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.add_u" (i32.const 0) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.add_u" (i32.const 0) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.add_u" (i32.const 0) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.add_u" (i32.const 0) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.add_u" (i32.const 0) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.sub

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.sub" (i32.const 0) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.sub" (i32.const 0) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.sub_u" (i32.const 0) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.sub_u" (i32.const 0) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.sub_u" (i32.const 0) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.sub_u" (i32.const 0) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.sub_u" (i32.const 0) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.and

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.and" (i32.const 0) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.and" (i32.const 0) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.and_u" (i32.const 0) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.and_u" (i32.const 0) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.and_u" (i32.const 0) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.and_u" (i32.const 0) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.and_u" (i32.const 0) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.or

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.or" (i32.const 0) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.or" (i32.const 0) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.or_u" (i32.const 0) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.or_u" (i32.const 0) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.or_u" (i32.const 0) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.or_u" (i32.const 0) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.or_u" (i32.const 0) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.xor

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.xor" (i32.const 0) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.xor" (i32.const 0) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.xor_u" (i32.const 0) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.xor_u" (i32.const 0) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.xor_u" (i32.const 0) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.xor_u" (i32.const 0) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.xor_u" (i32.const 0) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.xchg

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.xchg" (i32.const 0) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.xchg" (i32.const 0) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.xchg_u" (i32.const 0) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.xchg_u" (i32.const 0) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.xchg_u" (i32.const 0) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.xchg_u" (i32.const 0) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.xchg_u" (i32.const 0) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.cmpxchg (compare false)

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.cmpxchg" (i32.const 0) (i32.const 0) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.cmpxchg" (i32.const 0) (i64.const 0) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.cmpxchg_u" (i32.const 0) (i32.const 0) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.cmpxchg_u" (i32.const 0) (i32.const 0x11111111) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.cmpxchg_u" (i32.const 0) (i32.const 0) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.cmpxchg_u" (i32.const 0) (i32.const 0x11111111) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.cmpxchg_u" (i32.const 0) (i64.const 0) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.cmpxchg_u" (i32.const 0) (i64.const 0x1111111111111111) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.cmpxchg_u" (i32.const 0) (i64.const 0) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.cmpxchg_u" (i32.const 0) (i64.const 0x1111111111111111) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.cmpxchg_u" (i32.const 0) (i64.const 0) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.cmpxchg_u" (i32.const 0) (i64.const 0x1111111111111111) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

;; *.atomic.rmw*.cmpxchg (compare true)

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw.cmpxchg" (i32.const 0) (i32.const 0x11111111) (i32.const 0x12345678)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw.cmpxchg" (i32.const 0) (i64.const 0x1111111111111111) (i64.const 0x0101010102020202)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw8.cmpxchg_u" (i32.const 0) (i32.const 0x11) (i32.const 0xcdcdcdcd)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i32.atomic.rmw16.cmpxchg_u" (i32.const 0) (i32.const 0x1111) (i32.const 0xcafecafe)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw8.cmpxchg_u" (i32.const 0) (i64.const 0x11) (i64.const 0x4242424242424242)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw16.cmpxchg_u" (i32.const 0) (i64.const 0x1111) (i64.const 0xbeefbeefbeefbeef)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")

(invoke "init" (i64.const 0x1111111111111111))
(assert_trap (invoke "i64.atomic.rmw32.cmpxchg_u" (i32.const 0) (i64.const 0x11111111) (i64.const 0xcabba6e5cabba6e5)) "not implemented")
(assert_trap (invoke "i64.atomic.load" (i32.const 0)) "not implemented")