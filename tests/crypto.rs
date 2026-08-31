// 测试 SSO 密码 AES 加密；无需改参数，不依赖网络
use njupt::login::sso::{IAM_CHECK_KEY, iam_encrypt, iam_encrypt_default};

#[test]
fn iam_check_key_yields_aes128_material() {
    assert_eq!(format!("iam{IAM_CHECK_KEY}").len(), 16);
}

#[test]
fn iam_encrypt_round_shape() {
    let cipher = iam_encrypt_default("student").expect("encrypt");
    assert!(!cipher.is_empty());
    assert!(cipher.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(cipher, iam_encrypt("student", IAM_CHECK_KEY).unwrap());
}
