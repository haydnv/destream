use destream::FromStream;
use tokio_util::bytes::Bytes;

#[tokio::test]
async fn it_works() {
    #[derive(FromStream, Debug, PartialEq)]
    struct Foo {
        a: Option<i32>,
        b: Option<String>,
        _c: Option<f64>,
    }

    let s = r#"{"a":1,"b":"foo","c":1.23}"#.to_string();
    let stream = get_stream(s);
    let foo: Foo = destream_json::decode((), stream).await.unwrap();
    assert_eq!(
        foo,
        Foo {
            a: Some(1),
            b: Some("foo".to_string()),
            _c: Some(1.23)
        }
    );
}

fn get_stream(s: String) -> impl futures::Stream<Item = Bytes> + Send + Unpin {
    Box::pin(futures::stream::once(async move {
        Bytes::copy_from_slice(s.as_bytes())
    }))
}
