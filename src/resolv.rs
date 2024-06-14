use std::io::Write;

pub fn generate(
    mut output: Box<dyn Write>,
    suffixes: &[&str],
    servers: &[&str],
) -> anyhow::Result<()> {
    writeln!(output, "# Auto-generated resolv.conf file")?;
    writeln!(output, "#")?;
    writeln!(
        output,
        "# This uses the output of ipconfig.exe on the Windows side to determine"
    )?;
    writeln!(output, "# valid DNS servers and search suffixes.")?;
    writeln!(output)?;
    if !suffixes.is_empty() {
        write!(output, "search")?;
        for &suffix in suffixes.iter() {
            write!(output, " {suffix}")?;
        }
        writeln!(output)?;
    } else {
        writeln!(
            output,
            "# No DNS search suffixes found. Check your ipconfig.exe output!"
        )?;
    }
    writeln!(output)?;
    if !servers.is_empty() {
        for &dns_server in servers.iter() {
            writeln!(output, "nameserver {dns_server}")?;
        }
    } else {
        write!(
            output,
            "# No DNS nameservers found. Check your ipconfig.exe output!"
        )?;
    }
    output.flush()?;
    Ok(())
}
