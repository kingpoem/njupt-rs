// 测试业务客户端能否挂接 DirectTransport；无需改参数，不依赖网络
use njupt::card::Card;
use njupt::jwxt::Jwxt;
use njupt::library::Library;
use njupt::login::{DirectTransport, default_http_client};

#[test]
fn business_clients_accept_direct_transport() {
    let http = default_http_client().expect("http client");
    let jwxt = Jwxt::new(DirectTransport::new(http.clone()));
    let card = Card::new(DirectTransport::new(http.clone()));
    let library = Library::new(DirectTransport::new(http));

    let _ = (jwxt.transport(), card.transport(), library.transport());
}
