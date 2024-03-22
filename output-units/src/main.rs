use bstr::BString;
use output_units::OutputUnits;

fn main() {
    let mut units = OutputUnits::new();

    units.push("abc", "hi to ja");

    let mut out = Vec::new();

    units.encode(&mut out).unwrap();

    let str = BString::new(out);
    print!("{str}");
}
