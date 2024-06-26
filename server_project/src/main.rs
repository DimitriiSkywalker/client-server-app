use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use tokio::sync::{Mutex, Semaphore};
use std::sync::Arc;
use rand::Rng;
use std::time::Instant;
use tokio::signal;

// Структура для хранения статистики сервера
#[derive(Default)]
struct ServerStats {
    total_connections: u32,
    total_session_time: u128,
    max_session_time: u128,
    min_session_time: Option<u128>,
}

impl ServerStats {
    // Конструктор для создания новой структуры ServerStats
    fn new() -> Self {
        ServerStats {
            total_connections: 0,
            total_session_time: 0,
            max_session_time: 0,
            min_session_time: None,
        }
    }

    // Обновление статистики для текущей сессии
    fn update_session_stats(&mut self, session_time: u128) {
        self.total_connections += 1;
        self.total_session_time += session_time;
        self.max_session_time = self.max_session_time.max(session_time);
        self.min_session_time = match self.min_session_time {
            Some(min) => Some(min.min(session_time)),
            None => Some(session_time),
        };
    }

    // Вывод статистики
    fn print_stats(&self) {
        println!("Total connections: {}", self.total_connections);
        println!("Total session duration: {} ms", self.total_session_time);
        println!("Maximum session duration: {} ms", self.max_session_time);
        match self.min_session_time {
            Some(min) => println!("Minimum session duration: {} ms", min),
            None => println!("Minimum session duration is not set"),
        }
    }
}

// Обработчик запросов
async fn handle_request(stats: web::Data<Arc<Mutex<ServerStats>>>, semaphore: web::Data<Arc<Semaphore>>) -> impl Responder {
    let start_time = Instant::now();

    // Запрашиваем разрешение семафора
    let _permit = semaphore.acquire().await.expect("Semaphore error");

    // Симулируем время обработки запроса
    let processing_time: u64 = rand::thread_rng().gen_range(100..=500);
    tokio::time::sleep(tokio::time::Duration::from_millis(processing_time)).await;

    // Обновляем статистику сервера
    let mut stats = stats.lock().await;
    stats.update_session_stats(processing_time as u128);

    let elapsed = start_time.elapsed().as_millis();
    HttpResponse::Ok().body(format!("Request processed in {} ms", elapsed))
}

// Обработчик ping запросов
async fn ping_handler() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

// Функция запуска сервера
async fn run_server() -> std::io::Result<()> {
    // Создаём общие данные для статистики и семафора
    let stats = Arc::new(Mutex::new(ServerStats::new()));
    let semaphore = Arc::new(Semaphore::new(5)); // Семафор для ограничения количества одновременных запросов

    // Клонируем ссылки на статистику и семафор для передачи в замыкание
    let stats_ref = Arc::clone(&stats);
    let semaphore_ref = Arc::clone(&semaphore);

    // Создаём сервер Actix с обработчиками и переданными данными
    let server = HttpServer::new(move || {
        App::new()
            .data(stats_ref.clone())
            .data(semaphore_ref.clone())
            .route("/", web::get().to(handle_request))
            .route("/ping", web::get().to(ping_handler))
    })
    .bind("127.0.0.1:8080")? // Привязываем сервер к адресу и порту
    .run(); // Запускаем сервер

    // Ожидаем завершения сервера или получения сигнала прерывания
    let _ = tokio::select! {
        _ = server => {
            println!("Server has shut down.");
        }
        _ = signal::ctrl_c() => {
            println!("Ctrl-C received. Shutting down server...");
        }
    };

    // Выводим статистику сервера после его завершения
    let stats = stats.lock().await;
    stats.print_stats();

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_server().await // Запускаем сервер
}

