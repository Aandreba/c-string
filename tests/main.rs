#[cfg(feature = "alloc")]
use cstring::CString;

#[cfg(feature = "alloc")]
#[test]
fn main () {
    const AL: &CSubStr = unsafe { CSubStr::from_str_unchecked("al") };
    use cstring::CSubStr;

    let correct = CString::from_string("Alex!".to_string()).unwrap();
    let upper = correct.to_uppercase();
    let lower = correct.to_lowercase();

    println!("{correct}\n{upper}\n{lower}");
}