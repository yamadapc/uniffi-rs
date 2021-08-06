{% import "macros.kt" as kt %}
{%- let e = self.inner() %}

// Error {{ e.name() }}
{%- let toplevel_name=e.name()|exception_name_kt %}
sealed class {{ toplevel_name }}: Exception(){% if self.contains_object_references() %}, Disposable {% endif %} {
    // Each variant is a nested class
    {% for variant in e.variants() -%}
    {% if !variant.has_fields() -%}
    class {{ variant.name()|exception_name_kt }} : {{ toplevel_name }}()
    {% else %}
    class {{ variant.name()|exception_name_kt }}(
        {% for field in variant.fields() -%}
        val {{ field.name()|var_name_kt }}: {{ field.type_()|type_kt}}{% if loop.last %}{% else %}, {% endif %}
        {% endfor -%}
    ) : {{ toplevel_name }}()
    {%- endif %}
    {% endfor %}

    companion object ErrorHandler : CallStatusErrorHandler<{{ toplevel_name }}> {
        override fun lift(error_buf: RustBuffer.ByValue): {{ toplevel_name }} {
            return liftFromRustBuffer(error_buf) { error_buf -> read(error_buf) }
        }

        fun read(error_buf: ByteBuffer): {{ toplevel_name }} {
            return when(error_buf.getInt()) {
                {%- for variant in e.variants() %}
                {{ loop.index }} -> {{ toplevel_name }}.{{ variant.name()|exception_name_kt }}({% if variant.has_fields() %}
                    {% for field in variant.fields() -%}
                    {{ "error_buf"|read_kt(field.type_()) }}{% if loop.last %}{% else %},{% endif %}
                    {% endfor -%}
                {%- endif -%})
                {%- endfor %}
                else -> throw RuntimeException("invalid error enum value, something is very wrong!!")
            }
        }
    }

    {% if self.contains_object_references() %}
    @Suppress("UNNECESSARY_SAFE_CALL") // codegen is much simpler if we unconditionally emit safe calls here
    override fun destroy() {
        when(this) {
            {%- for variant in e.variants() %}
            is {{ e.name()|class_name_kt }}.{{ variant.name()|class_name_kt }} -> {
                {%- if variant.has_fields() %}
                {% call kt::destroy_fields(variant) %}
                {% else -%}
                // Nothing to destroy
                {%- endif %}
            }
            {%- endfor %}
        }.let { /* this makes the `when` an expression, which ensures it is exhaustive */ }
    }
    {% endif %}
}
