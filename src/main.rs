use tonic::{transport::Server, Request, Response, Status};

use blog::blog_server::{Blog, BlogServer};
use blog::{Post, PostRequest};
use tokio_postgres::{Client, NoTls};

mod blog {
    include!("blog.rs");
}

pub struct BlogImpl {
    client: Client,
}

#[tonic::async_trait]
impl Blog for BlogImpl {
    async fn get_post(&self, request: Request<PostRequest>) -> Result<Response<Post>, Status> {
        match self
            .client
            .query_one(
                "SELECT id, title, text, category, author FROM public.post WHERE id = $1;",
                &[&request.get_ref().id],
            )
            .await
        {
            Err(err) => Err(Status::internal(err.to_string())),
            Ok(row) => {
                let response = Post {
                    id: row.get(0),
                    title: row.get(1),
                    text: row.get(2),
                    category: row.get(3),
                    auhtor: row.get(4),
                };

                Ok(Response::new(response))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=127.0.0.1 port=5432 user=postgres password=admin dbname=postgres sslmode=disable",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(err) = connection.await {
            eprintln!("Connection error: {}", err);
        }
    });

    let addr = "[::1]:2021".parse().unwrap();

    let blog = BlogImpl { client };

    println!("Blog server listening on {}", addr);

    Server::builder()
        .add_service(BlogServer::new(blog))
        .serve(addr)
        .await?;

    Ok(())
}
