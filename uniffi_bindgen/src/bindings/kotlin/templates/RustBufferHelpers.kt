// Helpers for reading primitive data types from a bytebuffer.

internal fun<T> liftFromRustBuffer(rbuf: RustBuffer.ByValue, readItem: (ByteBuffer) -> T): T {
    val buf = rbuf.asByteBuffer()!!
    try {
       val item = readItem(buf)
       if (buf.hasRemaining()) {
           throw RuntimeException("junk remaining in buffer after lifting, something is very wrong!!")
       }
       return item
    } finally {
        RustBuffer.free(rbuf)
    }
}

internal fun<T> lowerIntoRustBuffer(v: T, writeItem: (T, RustBufferBuilder) -> Unit): RustBuffer.ByValue {
    // TODO: maybe we can calculate some sort of initial size hint?
    val buf = RustBufferBuilder()
    try {
        writeItem(v, buf)
        return buf.finalize()
    } catch (e: Throwable) {
        buf.discard()
        throw e
    }
}

// For every type used in the interface, we provide helper methods for conveniently
// lifting and lowering that type from C-compatible data, and for reading and writing
// values of that type in a buffer.

{% for typ in ci.iter_types() %}
{%- match typ|helper_code %}
{%- when Some with (code) %}
{{ code }}
{%- else %}
{% let canonical_type_name = typ.canonical_name()|class_name_kt %}
{%- match typ -%}

{% when Type::Duration -%}
{% let type_name = typ|type_kt %}

internal fun lift{{ canonical_type_name }}(rbuf: RustBuffer.ByValue): {{ type_name }} {
    return liftFromRustBuffer(rbuf) { buf ->
        read{{ canonical_type_name }}(buf)
    }
}

internal fun read{{ canonical_type_name }}(buf: ByteBuffer): {{ type_name }} {
    // Type mismatch (should be u64) but we check for overflow/underflow below
    val seconds = buf.getLong()
    // Type mismatch (should be u32) but we check for overflow/underflow below
    val nanoseconds = buf.getInt().toLong()
    if (seconds < 0) {
        throw java.time.DateTimeException("Duration exceeds minimum or maximum value supported by uniffi")
    }
    if (nanoseconds < 0) {
        throw java.time.DateTimeException("Duration nanoseconds exceed minimum or maximum supported by uniffi")
    }
    return {{ type_name }}.ofSeconds(seconds, nanoseconds)
}

internal fun lower{{ canonical_type_name }}(v: {{ type_name }}): RustBuffer.ByValue {
    return lowerIntoRustBuffer(v) { v, buf ->
        write{{ canonical_type_name }}(v, buf)
    }
}

internal fun write{{ canonical_type_name }}(v: {{ type_name }}, buf: RustBufferBuilder) {
    if (v.seconds < 0) {
        // Rust does not support negative Durations
        throw IllegalArgumentException("Invalid duration, must be non-negative")
    }

    if (v.nano < 0) {
        // Java docs provide guarantee that nano will always be positive, so this should be impossible
        // See: https://docs.oracle.com/javase/8/docs/api/java/time/Duration.html
        throw IllegalArgumentException("Invalid duration, nano value must be non-negative")
    }

    // Type mismatch (should be u64) but since Rust doesn't support negative durations we should be OK
    buf.putLong(v.seconds)
    // Type mismatch (should be u32) but since values will always be between 0 and 999,999,999 it should be OK
    buf.putInt(v.nano)
}

{% else %}
{# This type cannot be lifted, lowered or serialized (yet) #}
// XXX Type {{ typ|type_kt }} is in use, but not handled.
{%- endmatch %}
{%- endmatch %}

{% endfor %}
