use rolldown::Bundle;

#[test]
fn output_should_be_utf8() {
  let bundle = Bundle::new("tests/fixtures/side-effects/main.js").expect("Create bundle failed");
  let mut output = Vec::new();
  let mut sm = Vec::new();
  assert!(bundle.generate(&mut output, Some(&mut sm)).is_ok());
  assert!(String::from_utf8(output).is_ok());
}
#[cfg(test)]
mod side_effects {
  use super::*;
  #[test]
  fn write() {
    let bundle = Bundle::new("tests/fixtures/side-effects/main.js").expect("Create bundle failed");
    let mut output = Vec::new();
    let mut sm = Vec::new();
    bundle.generate(&mut output, Some(&mut sm)).unwrap();
    let codes = String::from_utf8(output).unwrap();
    println!("codes {:?}", codes);
    assert!(codes.contains(r"add.name = 'Function(add)'"));
  }

  #[test]
  fn io() {
    let bundle = Bundle::new("tests/fixtures/side-effects/main.js").expect("Create bundle failed");
    let mut output = Vec::new();
    let mut sm = Vec::new();
    bundle.generate(&mut output, Some(&mut sm)).unwrap();
    let codes = String::from_utf8(output).unwrap();
    assert!(codes.contains(r"console.log(add.name)"));
  }
}
