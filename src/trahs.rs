use r2d2::Pool;
use tonic::{transport::Server, Request, Response, Status};

use blog::blog_server::{Blog, BlogServer};
use blog::{Post, PostRequest};

use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};

mod blog {
    include!("blog.rs");
}

pub struct BlogImpl {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

#[tonic::async_trait]
impl Blog for BlogImpl {
    async fn get_post(&self, request: Request<PostRequest>) -> Result<Response<Post>, Status> {
        println!("Requestfrom {:?}", request.remote_addr());
        // let id = request.get_ref().id;
        let pool =self.pool.get();
       match tokio::task::spawn_blocking(move ||{
            match pool {
                Err(err) => Err(Status::internal("internal server error bro".to_string())),
                Ok(mut client) => {
                    match client.query_one(
                        "SELECT id, title, text, category, author FROM public.post WHERE id = $1;",
                        &[&request.get_ref().id],
                    ) {
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
        }).await{
            Err(_)=>Err(Status::internal("async fucked".to_string())),
            Ok(res)=>res,
        }
        // match self.pool.get() {
        //     Err(err) => Err(Status::internal("internal server error bro".to_string())),
        //     Ok(mut client) => {
        //         match client.query_one(
        //             "SELECT id, title, text, category, author FROM public.post WHERE id = $1;",
        //             &[&request.get_ref().id],
        //         ) {
        //             Err(err) => Err(Status::internal(err.to_string())),
        //             Ok(row) => {
        //                 let response = Post {
        //                     id: row.get(0),
        //                     title: row.get(1),
        //                     text: row.get(2),
        //                     category: row.get(3),
        //                     auhtor: row.get(4),
        //                 };

        //                 Ok(Response::new(response))
        //             }
        //         }
        //     }
        // }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager: PostgresConnectionManager<NoTls> = PostgresConnectionManager::new(
    // let manager= PostgresConnectionManager::new(
        "host=127.0.0.1 port=5432 user=postgres password=admin dbname=postgres sslmode=disable"
            .parse()
            .unwrap(),
        NoTls,
    );

    let pool: Pool<PostgresConnectionManager<NoTls>> = r2d2::Pool::new(manager)?;
    // let pool = r2d2::Pool::new(manager)?;
    // let id:i64 = 6;

    // match pool.get() {
    //     Err(err) =>println!("error"),
    //     Ok(mut client) => {
    //         match client.query_one(
    //             "SELECT id, title, text, category, author FROM public.post WHERE id = $1;",
    //             &[&id],
    //         ) {
    //             Err(err) => println!("erro r"),
    //             Ok(row) => {
    //                 let response = Post {
    //                     id: row.get(0),
    //                     title: row.get(1),
    //                     text: row.get(2),
    //                     category: row.get(3),
    //                     auhtor: row.get(4),
    //                 };

    //                 println!("text: {}",response.text);
    //             }
    //         }
    //     }
    // }
    
    let addr = "[::1]:2021".parse().unwrap();

    let blog = BlogImpl { pool };

    println!("Blog server listening on {}", addr);

    Server::builder()
        .add_service(BlogServer::new(blog))
        .serve(addr)
        .await?;

    Ok(())
}
