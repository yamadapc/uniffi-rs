// This file was autogenerated by some hot garbage in the `uniffi` crate.
// Trust me, you don't want to mess with it!

@file:Suppress("NAME_SHADOWING")

package {{ config.package_name() }};

// Common helper code.
//
// Ideally this would live in a separate .kt file where it can be unittested etc
// in isolation, and perhaps even published as a re-useable package.
//
// However, it's important that the detils of how this helper code works (e.g. the
// way that different builtin types are passed across the FFI) exactly match what's
// expected by the Rust code on the other side of the interface. In practice right
// now that means coming from the exact some version of `uniffi` that was used to
// compile the Rust component. The easiest way to ensure this is to bundle the Kotlin
// helpers directly inline like we're doing here.

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference
import java.util.concurrent.locks.ReentrantLock
import kotlin.concurrent.withLock

{% include "RustBufferTemplate.kt" %}

{% include "RustBufferHelpers.kt" %}

{% include "NamespaceLibraryTemplate.kt" %}

{% include "Helpers.kt" %}

// Public interface members begin here.


{% for m in self.members() %}
{%- match m.definition_code(oracle) %}
{% when Some with (code) %}
{{ code }}
{% else %}
{% endmatch %}
{%- endfor -%}

// Error definitions
{% include "ErrorTemplate.kt" %}

// Public facing records
{%- for rec in ci.iter_record_definitions() %}
{% include "RecordTemplate.kt" %}
{% endfor %}

// Namespace functions
{% for func in ci.iter_function_definitions() %}
{% include "TopLevelFunctionTemplate.kt" %}
{% endfor %}

// Objects
{% for obj in ci.iter_object_definitions() %}
{% include "ObjectTemplate.kt" %}
{% endfor %}

// Callback Interfaces
{% for cbi in ci.iter_callback_interface_definitions() %}
{% include "CallbackInterfaceTemplate.kt" %}
{% endfor %}

{% import "macros.kt" as kt %}
