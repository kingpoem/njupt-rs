// 测试公共辅助：从项目根目录 .env 读取 NJUPT_USERNAME / NJUPT_PASSWORD
use std::path::Path;

use njupt::Credentials;

pub fn creds() -> Credentials {
    let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenvy::from_path(&env_path);

    let username = std::env::var("NJUPT_USERNAME").unwrap_or_default();
    let password = std::env::var("NJUPT_PASSWORD").unwrap_or_default();
    assert!(
        !username.is_empty() && !password.is_empty(),
        "请在项目根目录 .env 中填写 NJUPT_USERNAME / NJUPT_PASSWORD（可参考 .env.example）"
    );
    Credentials::new(username, password)
}
