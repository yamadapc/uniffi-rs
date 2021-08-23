// ==============================================================================
// Generated from CallbackInterfaceTemplate.swift
// ==============================================================================

{% let type_name = cbi.name()|class_name_swift %}
public protocol {{ type_name }}: AnyObject {
    {% for meth in cbi.methods() -%}
    func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %})
    {% call swift::throws(meth) %}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_swift -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

fileprivate extension {{ type_name }} {
    fileprivate typealias FfiType = UnsafeMutableRawPointer

    fileprivate static func read(from buf: Reader) throws -> Self {
        let v: UInt64 = try buf.readInt()
        // The Rust code won't compile if a pointer won't fit in a UInt64.
        // We have to go via `UInt` because that's the thing that's the size of a pointer.
        let ptr = UnsafeMutableRawPointer(bitPattern: UInt(truncatingIfNeeded: v))
        if (ptr == nil) {
            throw UniffiInternalError.unexpectedNullPointer
        }
        return try self.lift(ptr!)
    }

    fileprivate func write(into buf: Writer) {
        // This fiddling is because `Int` is the thing that's the same size as a pointer.
        // The Rust code won't compile if a pointer won't fit in a `UInt64`.
        buf.writeInt(UInt64(bitPattern: Int64(Int(bitPattern: self.lower()))))
    }

     fileprivate static func lift(_ pointer: UnsafeMutableRawPointer) throws -> Self {
         return Unmanaged.fromOpaque(pointer).takeUnretainedValue()
     }

    fileprivate func lower() -> UnsafeMutableRawPointer {
        // return self.pointer
        return Unmanaged.passRetained(self).toOpaque()
    }
}
