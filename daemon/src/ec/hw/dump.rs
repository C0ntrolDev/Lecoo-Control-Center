use anyhow::Result;
use super::EcDevice;
use crate::ec::Addr;

/// Human-readable snapshot of profile + runtime + live register values.
/// Read-only: safe to run on any board, including forced --profile guesses.
pub fn dump_profile(ec: &EcDevice) -> Result<String> {
    let mut out = String::new();
    let p = ec.profile;

    out.push_str(&format!("board:        {}\n", p.id));
    out.push_str(&format!("daemon:       {}\n", crate::VERSION));
    out.push_str(&format!("dmi match:    {:?}\n", p.dmi));
    out.push_str(&format!("port:         {:#04X}\n", ec.rt.port));
    out.push_str(&format!("hram window:  {:#06X}  (candidates {:04X?})\n",
        ec.rt.hram_offset, p.hram_candidates));
    out.push_str(&format!("chip:         IT{:02X}{:02X}-{:02X}\n",
        ec.rt.chip_id1, ec.rt.chip_id2, ec.rt.chip_ver));
    out.push_str(&format!("charge:       {:?}\n", p.charge));
    out.push_str(&format!("kbd:          {:?}\n", p.kbd));
    out.push_str(&format!("led:          {:?}\n\n", p.led));

    out.push_str("name                     addr        resolved  value\n");
    for (name, addr) in p.addr_map() {
        let kind = match addr {
            Addr::Reg(x) => format!("Reg({x:#06X})"),
            Addr::Ram(x) => format!("Ram({x:#04X})"),
            Addr::Banked(x) => format!("Bank({x:#06X})"),
        };
        let resolved = match addr {
            Addr::Reg(x) => x,
            Addr::Ram(x) => ec.rt.hram_offset + x,
            Addr::Banked(x) => x + (ec.rt.hram_offset & 0xF000),
        };
        let value = match ec.read(addr) {
            Ok(v) => format!("{v:#04X}"),
            Err(_) => "err".into(),
        };
        out.push_str(&format!("{name:<24} {kind:<11} {resolved:#06X}    {value}\n"));
    }
    Ok(out)
}
