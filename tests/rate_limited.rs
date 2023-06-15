use proxy_wasm_test_framework::tester;
use proxy_wasm_test_framework::types::{Action, BufferType, LogLevel, MapType, ReturnType};
use serial_test::serial;

#[test]
#[serial]
fn it_loads() {
    let args = tester::MockSettings {
        wasm_path: "target/wasm32-unknown-unknown/release/wasm_shim.wasm".to_string(),
        quiet: false,
        allow_unexpected: false,
    };
    let mut module = tester::mock(args).unwrap();

    module
        .call_start()
        .execute_and_expect(ReturnType::None)
        .unwrap();

    let root_context = 1;
    let cfg = r#"{
        "failure_mode_deny": true,
        "rate_limit_policies": []
    }"#;

    module
        .call_proxy_on_context_create(root_context, 0)
        .expect_log(Some(LogLevel::Info), Some("set_root_context #1"))
        .execute_and_expect(ReturnType::None)
        .unwrap();
    module
        .call_proxy_on_configure(root_context, 0)
        .expect_log(Some(LogLevel::Info), Some("on_configure #1"))
        .expect_get_buffer_bytes(Some(BufferType::PluginConfiguration))
        .returning(Some(cfg.as_bytes()))
        .expect_log(Some(LogLevel::Info), None)
        .execute_and_expect(ReturnType::Bool(true))
        .unwrap();

    let http_context = 2;
    module
        .call_proxy_on_context_create(http_context, root_context)
        .expect_log(Some(LogLevel::Info), Some("create_http_context #2"))
        .execute_and_expect(ReturnType::None)
        .unwrap();

    module
        .call_proxy_on_request_headers(http_context, 0, false)
        .expect_log(Some(LogLevel::Info), Some("on_http_request_headers #2"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":authority"))
        .returning(Some("cars.toystore.com"))
        .expect_log(
            Some(LogLevel::Info),
            Some("context #2: Allowing request to pass because zero descriptors generated"),
        )
        .execute_and_expect(ReturnType::Action(Action::Continue))
        .unwrap();

    module
        .call_proxy_on_response_headers(http_context, 0, false)
        .execute_and_expect(ReturnType::Action(Action::Continue))
        .unwrap();
}

#[test]
#[serial]
fn it_limits() {
    let args = tester::MockSettings {
        wasm_path: "target/wasm32-unknown-unknown/release/wasm_shim.wasm".to_string(),
        quiet: false,
        allow_unexpected: false,
    };
    let mut module = tester::mock(args).unwrap();

    module
        .call_start()
        .execute_and_expect(ReturnType::None)
        .unwrap();

    let root_context = 1;
    let cfg = r#"{
        "failure_mode_deny": true,
        "rate_limit_policies": [
                {
            "name": "some-name",
            "rate_limit_domain": "RLS-domain",
            "upstream_cluster": "limitador-cluster",
            "hostnames": ["*.toystore.com", "example.com"],
            "gateway_actions": [
            {
                "rules": [
                {
                    "paths": ["/admin/toy"],
                    "hosts": ["cars.toystore.com"],
                    "methods": ["POST"]
                }],
                "configurations": [
                {
                    "actions": [
                    {
                        "generic_key": {
                            "descriptor_key": "admin",
                            "descriptor_value": "1"
                        }
                    }
                    ]
                }
                ]
            }
            ]
        }
        ]
    }"#;

    module
        .call_proxy_on_context_create(root_context, 0)
        .expect_log(Some(LogLevel::Info), Some("set_root_context #1"))
        .execute_and_expect(ReturnType::None)
        .unwrap();
    module
        .call_proxy_on_configure(root_context, 0)
        .expect_log(Some(LogLevel::Info), Some("on_configure #1"))
        .expect_get_buffer_bytes(Some(BufferType::PluginConfiguration))
        .returning(Some(cfg.as_bytes()))
        .expect_log(Some(LogLevel::Info), None)
        .execute_and_expect(ReturnType::Bool(true))
        .unwrap();

    let http_context = 2;
    module
        .call_proxy_on_context_create(http_context, root_context)
        .expect_log(Some(LogLevel::Info), Some("create_http_context #2"))
        .execute_and_expect(ReturnType::None)
        .unwrap();

    module
        .call_proxy_on_request_headers(http_context, 0, false)
        .expect_log(Some(LogLevel::Info), Some("on_http_request_headers #2"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":authority"))
        .returning(Some("cars.toystore.com"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":path"))
        .returning(Some("/admin/toy"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":method"))
        .returning(Some("POST"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":authority"))
        .returning(Some("cars.toystore.com"))
        .expect_grpc_call(
            Some("limitador-cluster"),
            Some("envoy.service.ratelimit.v3.RateLimitService"),
            Some("ShouldRateLimit"),
            Some(&[0, 0, 0, 0]),
            Some(&[
                10, 10, 82, 76, 83, 45, 100, 111, 109, 97, 105, 110, 18, 12, 10, 10, 10, 5, 97,
                100, 109, 105, 110, 18, 1, 49, 24, 1,
            ]),
            Some(5000),
        )
        .returning(Some(42))
        .expect_log(
            Some(LogLevel::Info),
            Some("Initiated gRPC call (id# 42) to Limitador"),
        )
        .execute_and_expect(ReturnType::Action(Action::Pause))
        .unwrap();

    let grpc_response: [u8; 2] = [8, 1];
    module
        .call_proxy_on_grpc_receive(http_context, 42, grpc_response.len() as i32)
        .expect_log(
            Some(LogLevel::Info),
            Some("received gRPC call response: token: 42, status: 0"),
        )
        .expect_get_buffer_bytes(Some(BufferType::GrpcReceiveBuffer))
        .returning(Some(&grpc_response))
        .execute_and_expect(ReturnType::None)
        .unwrap();

    module
        .call_proxy_on_response_headers(http_context, 0, false)
        .execute_and_expect(ReturnType::Action(Action::Continue))
        .unwrap();
}

#[test]
#[serial]
fn it_passes_additional_headers() {
    let args = tester::MockSettings {
        wasm_path: "target/wasm32-unknown-unknown/release/wasm_shim.wasm".to_string(),
        quiet: false,
        allow_unexpected: false,
    };
    let mut module = tester::mock(args).unwrap();

    module
        .call_start()
        .execute_and_expect(ReturnType::None)
        .unwrap();

    let root_context = 1;
    let cfg = r#"{
        "failure_mode_deny": true,
        "rate_limit_policies": [
                {
            "name": "some-name",
            "rate_limit_domain": "RLS-domain",
            "upstream_cluster": "limitador-cluster",
            "hostnames": ["*.toystore.com", "example.com"],
            "gateway_actions": [
            {
                "rules": [
                {
                    "paths": ["/admin/toy"],
                    "hosts": ["cars.toystore.com"],
                    "methods": ["POST"]
                }],
                "configurations": [
                {
                    "actions": [
                    {
                        "generic_key": {
                            "descriptor_key": "admin",
                            "descriptor_value": "1"
                        }
                    }
                    ]
                }
                ]
            }
            ]
        }
        ]
    }"#;

    module
        .call_proxy_on_context_create(root_context, 0)
        .expect_log(Some(LogLevel::Info), Some("set_root_context #1"))
        .execute_and_expect(ReturnType::None)
        .unwrap();
    module
        .call_proxy_on_configure(root_context, 0)
        .expect_log(Some(LogLevel::Info), Some("on_configure #1"))
        .expect_get_buffer_bytes(Some(BufferType::PluginConfiguration))
        .returning(Some(cfg.as_bytes()))
        .expect_log(Some(LogLevel::Info), None)
        .execute_and_expect(ReturnType::Bool(true))
        .unwrap();

    let http_context = 2;
    module
        .call_proxy_on_context_create(http_context, root_context)
        .expect_log(Some(LogLevel::Info), Some("create_http_context #2"))
        .execute_and_expect(ReturnType::None)
        .unwrap();

    module
        .call_proxy_on_request_headers(http_context, 0, false)
        .expect_log(Some(LogLevel::Info), Some("on_http_request_headers #2"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":authority"))
        .returning(Some("cars.toystore.com"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":path"))
        .returning(Some("/admin/toy"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":method"))
        .returning(Some("POST"))
        .expect_get_header_map_value(Some(MapType::HttpRequestHeaders), Some(":authority"))
        .returning(Some("cars.toystore.com"))
        .expect_grpc_call(
            Some("limitador-cluster"),
            Some("envoy.service.ratelimit.v3.RateLimitService"),
            Some("ShouldRateLimit"),
            Some(&[0, 0, 0, 0]),
            Some(&[
                10, 10, 82, 76, 83, 45, 100, 111, 109, 97, 105, 110, 18, 12, 10, 10, 10, 5, 97,
                100, 109, 105, 110, 18, 1, 49, 24, 1,
            ]),
            Some(5000),
        )
        .returning(Some(42))
        .expect_log(
            Some(LogLevel::Info),
            Some("Initiated gRPC call (id# 42) to Limitador"),
        )
        .execute_and_expect(ReturnType::Action(Action::Pause))
        .unwrap();

    let grpc_response: [u8; 45] = [
        8, 1, 26, 18, 10, 4, 116, 101, 115, 116, 18, 10, 115, 111, 109, 101, 32, 118, 97, 108, 117,
        101, 26, 21, 10, 5, 111, 116, 104, 101, 114, 18, 12, 104, 101, 97, 100, 101, 114, 32, 118,
        97, 108, 117, 101,
    ];
    module
        .call_proxy_on_grpc_receive(http_context, 42, grpc_response.len() as i32)
        .expect_log(
            Some(LogLevel::Info),
            Some("received gRPC call response: token: 42, status: 0"),
        )
        .expect_get_buffer_bytes(Some(BufferType::GrpcReceiveBuffer))
        .returning(Some(&grpc_response))
        .execute_and_expect(ReturnType::None)
        .unwrap();

    module
        .call_proxy_on_response_headers(http_context, 0, false)
        .expect_add_header_map_value(
            Some(MapType::HttpResponseHeaders),
            Some("test"),
            Some("some value"),
        )
        .expect_add_header_map_value(
            Some(MapType::HttpResponseHeaders),
            Some("other"),
            Some("header value"),
        )
        .execute_and_expect(ReturnType::Action(Action::Continue))
        .unwrap();
}
