use ksni::blocking::TrayMethods;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zbus::{Connection, OwnedObjectPath, OwnedVariant, Proxy, Variant};

struct Watcher(Arc<Mutex<Option<String>>>);
#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl Watcher {
    fn register_status_notifier_item(&self, service: &str) {
        *self.0.lock().unwrap() = Some(service.to_owned());
    }
    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }
    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }
}
struct Tray(Arc<Mutex<Vec<String>>>);
impl ksni::Tray for Tray {
    fn id(&self) -> String {
        "arkret-zbus6-test".into()
    }
    fn title(&self) -> String {
        "Wire integration".into()
    }
    fn activate(&mut self, x: i32, y: i32) {
        self.0.lock().unwrap().push(format!("activate:{x}:{y}"));
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![ksni::menu::StandardItem {
            label: "Open".into(),
            activate: Box::new(|tray: &mut Self| tray.0.lock().unwrap().push("menu".into())),
            ..Default::default()
        }
        .into()]
    }
}
#[test]
fn blocking_tray_registers_and_serves_sni_menu_over_zbus6() {
    let service = Arc::new(Mutex::new(None));
    let watcher = zbus::block_on(async {
        zbus::connection::Builder::session()
            .name("org.kde.StatusNotifierWatcher")
            .serve_at("/StatusNotifierWatcher", Watcher(service.clone()))
            .build()
            .await
            .unwrap()
    });
    let callbacks = Arc::new(Mutex::new(Vec::new()));
    let handle = Tray(callbacks.clone()).spawn().unwrap();
    let name = service
        .lock()
        .unwrap()
        .clone()
        .expect("watcher registration");
    zbus::block_on(async {
        let connection = Connection::session().await.unwrap();
        let sni = Proxy::new(
            &connection,
            name.as_str(),
            "/StatusNotifierItem",
            "org.kde.StatusNotifierItem",
        )
        .await
        .unwrap();
        assert_eq!(
            sni.get_property::<String>("Id").await.unwrap(),
            "arkret-zbus6-test"
        );
        assert_eq!(
            sni.get_property::<String>("Title").await.unwrap(),
            "Wire integration"
        );
        let path: OwnedObjectPath = sni.get_property("Menu").await.unwrap();
        assert_eq!(path.as_str(), "/MenuBar");
        sni.call::<_, _, ()>("Activate", &(12i32, 34i32))
            .await
            .unwrap();
        let menu = Proxy::new(
            &connection,
            name.as_str(),
            path.as_str(),
            "com.canonical.dbusmenu",
        )
        .await
        .unwrap();
        type Layout = (i32, HashMap<String, OwnedVariant>, Vec<OwnedVariant>);
        let (_, layout): (u32, Layout) = menu
            .call("GetLayout", &(0i32, -1i32, Vec::<&str>::new()))
            .await
            .unwrap();
        assert_eq!(layout.0, 0);
        assert_eq!(layout.2.len(), 1);
        menu.call::<_, _, ()>("Event", &(1i32, "clicked", Variant::from(0i32), 0u32))
            .await
            .unwrap();
    });
    assert_eq!(*callbacks.lock().unwrap(), ["activate:12:34", "menu"]);
    handle.shutdown().wait();
    zbus::block_on(watcher.close()).unwrap();
}
