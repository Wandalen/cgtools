# Invariant: SVG Injection-Safe Output

### Scope

- **Purpose**: Guarantee that caller-controlled string content cannot break out of its intended SVG text or attribute context to inject markup or scripts.
- **Responsibility**: Document exactly which caller-controlled inputs are sanitized, how, and the one explicit case that is deliberately out of scope.
- **In Scope**: `Char`-stream text content and `ImageSource::Path` href values emitted by the SVG backend.
- **Out of Scope**: The contents of `ImageSource::Encoded` bytes when they are themselves SVG images — see Violation Consequences.

### Invariant Statement

Every caller-controlled string that the SVG backend places into text PCDATA or an XML attribute value is escaped or encoded before emission, so the SVG backend cannot be used to inject script tags, event-handler attributes, or other markup into the generated document through ordinary text or path-name input.

### Enforcement Mechanism

- **Text content**: `xml_text_escape` (`src/adapters/svg.rs`) entity-escapes the five XML-significant characters in `Char`-stream text — `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`, `"` → `&quot;`, `'` → `&apos;` — before the text is written into a `<text>` element.
- **Path-sourced image hrefs**: `path_to_href` (`src/adapters/svg.rs`) percent-encodes every byte of an `ImageSource::Path` value outside the RFC 3986 unreserved set plus `/` (i.e. only `A-Z a-z 0-9 - _ . ~ /` pass through unencoded), and normalizes Windows backslashes to forward slashes. Percent-encoding `"`, `<`, `>`, and `&` neutralizes attribute-injection payloads in the same pass that makes the path a valid URI reference.
- Both mechanisms are covered by dedicated tests: `text_escapes_xml_special_characters` and `image_path_escapes_attribute_injection` (the latter specifically asserts a `"` `onload="alert(1)"` payload cannot break out of the `href` attribute).

### Violation Consequences

**Scope limitation, not a violation**: this guarantee explicitly does not extend to `ImageSource::Encoded` bytes that are themselves SVG content. Those bytes are base64-embedded as-is inside a `data:image/svg+xml` `<image>` element — the backend has no visibility into their internal structure to sanitize, and a browser may execute `<script>` or event-handler content inside an embedded SVG image in some rendering contexts. A caller that accepts `ImageSource::Encoded` SVG bytes from an untrusted source is responsible for trusting or sanitizing that source itself; the backend's escaping only covers the two paths listed under Enforcement Mechanism. Should a future code path place caller-controlled text into a new attribute or content position without routing it through `xml_text_escape` / `path_to_href`, that path would silently reopen an injection vector with no test currently guarding it.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md) | Sole backend that emits caller-controlled strings into a markup document; this invariant is SVG-specific |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/svg.rs` | `xml_text_escape` (text PCDATA) and `path_to_href` (attribute percent-encoding) |

### Tests

| File | Relationship |
|------|--------------|
| `src/adapters/svg.rs` (inline `#[cfg(test)]`) | `text_escapes_xml_special_characters`, `image_path_escapes_attribute_injection` |
