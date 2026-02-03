(module
  (import "ic0" "msg_reply" (func $msg_reply))
  (import "ic0" "msg_reply_data_append" (func $msg_reply_data_append (param i32 i32)))

  (memory (export "memory") 1)
  (data (i32.const 0) "\00\00\00\00") ;; A 32-bit counter at memory address 0

  (func $read (export "read")
    ;; Reply with the 4 bytes of the counter
    (call $msg_reply_data_append
      (i32.const 0) ;; src: memory address 0
      (i32.const 4) ;; size: 4 bytes
    )
    (call $msg_reply)
  )

  (func $inc (export "inc")
    ;; Load current value from memory
    (i32.load (i32.const 0))
    ;; Increment it
    (i32.const 1)
    (i32.add)
    ;; Store the new value back to memory
    (i32.store (i32.const 0))
    ;; Reply
    (call $msg_reply)
  )

  ;; Also export with the wasmtime prefix for compatibility, though it won't work there.
  (export "canister_update inc" (func $inc))
  (export "canister_update read" (func $read))
)