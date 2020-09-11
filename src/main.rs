use actix_rt::System;
use actix_web::{web, get, App, HttpResponse, HttpServer, Responder, HttpRequest, Either, Error};
use std::thread;
use std::sync::mpsc;



//a request handler is an async function that accepts 0 to 10 parameters that can
//be extracted from a request (ie impl FromRequest) and returns a type that can be converted into
//an HttpResponse.
//Handler parameters are referred to as "Extractors" and there are many. You can have up to 10,
//and ordering doesn't matter. This seems annoying at first glance.
//	Path, Query, Json, Form, Data (Application State), HttpRequest, String bytes::Bytes, Payload
//more on extractors here: https://actix.rs/docs/extractors/

async fn example(_req: HttpRequest) -> HttpResponse {
	HttpResponse::Ok().body("Hello world")
}

//in some cases you need to respond more than one type of response. Use Either:
type RegisterResult = Either<HttpResponse, Result<&'static str, Error>>;
async fn either_example() -> RegisterResult {
	if 1 == 1 {
		Either::A(HttpResponse::BadRequest().body("Bad Data"))
	} else {
		Either::B(Ok("Hello!"))
	}
}

//For a real project, check out iplementing the Responder trait


//write a couple handlers:
async fn index(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
	HttpResponse::Ok().body(format!("Hello World {}", data.app_name))
}

async fn index2() -> impl Responder {
	HttpResponse::Ok().body("Hello world again!")
}

//Another option is to use Attribute Macros to define routes:
#[get("/hello")]
async fn index3() -> impl Responder {
	HttpResponse::Ok().body("Attribute macro get")
}





//Ensure using actix-rt as a dependency for this
//this actix_rt::main macro executes the associated function in the actix runtime. any async
//function can be marked and executed with this macro
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //then start the Http Server

	HttpServer::new(|| {

		//Add an application to this Http Server
		//App is used for registering routes for resources and middlewares
		//as well as storing shared state
		App::new()
			.data(AppState { app_name: String::from("Actix_state") })
			.route("/", web::get().to(index))
			.route("/again", web::get().to(index2))

		//and register the attribute macro as a service:
			.service(index3)

		//We can also register a service that puts forth a scope
			.service(
				//This will reply "scoped response" at /app/app_path
				web::scope("/app").route("/app_path", web::get().to(|| HttpResponse::Ok().body("Scoped response")))
			)

			.configure(config)

		//you can additionally check out guards which can handle routes based on
		//whether some conditions is met. I'll get back to guards later.

	}).bind("127.0.0.1:8888")?
		.run()
		.await


	// HttpServer accepts an application factory as a parameter which must have Send + Sync.
	// bind() must be used to bind to a specific socket address
	// to bind ssl socket, use bind_openssl() or bind_rustls()
	//
	// run() returns an instance of the server type. more methods exist for managing the server:
	// 	pause() pauses accepting connections
	// 	resume() resumes accepting incoming connections
	// 	stop() stops incoming connection processing, stop all workers and exit
	//
	// HttpServer automatically starts a number of workers equal to the logical
	// CPUs in the system. You can specify the number with workers()
	//
	// Each works receives its own application instance.


	//another way to write the server:
	// let (tx, rx) = mpsc::channel();
	//
	// thread::spawn(move || {
	// 	let sys = System::new("http-server");
	//
	// 	let srv = HttpServer::new(|| {
	// 		App::new().route("/", web::get().to(|| HttpResponse::Ok()))
	// 	})
	// 		.bind("127.0.0.1:8888").unwrap()
	// 		.shutdown_timeout(60)
	// 		.run();
	//
	// 	let _ = tx.send(srv);
	// 	sys.run()
	// });
	//
	// let srv = rx.recv().unwrap();
	//
	// srv.pause().await;
	// srv.resume().await;
	// srv.stop(true).await;
}

//Some state struct
//HttpServer constructs an Application instance in each thread. So if you want shared state across
//threads, a Shareable object should be used (Send + Sync)
//Internally, web::Data uses uses Arc, so to avoid creating two Arcs, you should
//create data before registering it. use App::app_data for shared mutable state instead of App::data
struct AppState {
	app_name: String
}

//additionally, you can configure in a function
//this function can be located in a different module:
fn config(cfg: &mut web::ServiceConfig) {
	cfg.service(
		web::resource("/conf")
			.route(web::get().to(|| HttpResponse::Ok().body("config route")))
	);
}