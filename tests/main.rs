#[cfg(feature = "alloc")]
use cstring::CString;

#[cfg(feature = "alloc")]
#[test]
fn main () {
    const AL: &CSubStr = unsafe { CSubStr::from_str_unchecked("al") };
    use cstring::CSubStr;

    let mut correct = CString::from_string("Alex!".to_string()).unwrap();
    correct.uppercase();

    println!("{correct:?}");
}