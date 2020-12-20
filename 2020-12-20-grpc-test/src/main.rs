use anyhow::*;
use std::net::SocketAddr;
use tonic::{transport::Server, Request, Response, Status};
use self::hello_world::*;
use self::hello_world::greeter_server::*;
use self::hello_world::greeter_client::*;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

struct GrpcTest {
    count: std::sync::Mutex<i32>,
}

#[tonic::async_trait]
impl Greeter for GrpcTest {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>
    ) -> Result<Response<HelloReply>, Status> {
        let message = format!("You said: {}", request.get_ref().name);
        Ok(Response::new(HelloReply { message }))
    }

    async fn get_count(
        &self,
        request: Request<CountRequest>
    ) -> Result<Response<CountReply>, Status> {
        let mut count_guard = self.count.lock().unwrap();
        *count_guard += 1;
        Ok(Response::new(CountReply { count: *count_guard }))
    }
}

impl Default for GrpcTest {
    fn default() -> Self {
        GrpcTest {
            count: std::sync::Mutex::new(0),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let grpc_test = GrpcTest::default();

    let server = async {
        Server::builder()
            .add_service(GreeterServer::new(grpc_test))
            .serve(addr)
            .await
            .context("Server error")
    };
    let client = async {
        tokio::time::delay_for(std::time::Duration::from_millis(500)).await;
        let mut client = GreeterClient::connect("http://127.0.0.1:3000").await?;
        let response = client.say_hello(HelloRequest {
            name: "Alice".into(),
        }).await.context("say_hello client call failed")?;
        println!("response == {:?}", response);

        for i in 0..10_u32 {
            tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
            let response = client.get_count(CountRequest {}).await
                .with_context(|| format!("get_count, i == {}", i))?;
            println!("response #{} == {:?}", i, response);
        }
        Ok(())
    };
    tokio::try_join!(server, client)?;
    Ok(())
}
