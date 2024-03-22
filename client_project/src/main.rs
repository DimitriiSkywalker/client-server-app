use reqwest::Client;
use std::time::{Instant, Duration};
use futures::future::join_all;

// Функция для асинхронной проверки доступности сервера по URL
async fn is_server_available(url: &str) -> bool {
    let ping_url = format!("{}/ping", url);
    match reqwest::get(&ping_url).await {
        Ok(response) => response.status().is_success(), // Возвращаем true, если ответ успешный
        Err(_) => false, // Возвращаем false в случае ошибки
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_url = "http://127.0.0.1:8080";

    // Проверяем доступность сервера
    if !is_server_available(server_url).await {
        eprintln!("Error: Server {} is not available", server_url);
        std::process::exit(1); // Выходим из программы с ошибкой
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <number_of_requests>\n pass the required number of requests when launching the application, e.g.: cargo run 13", args[0]);
        std::process::exit(1); // Выходим из программы с ошибкой
    }

    let n_requests: usize = args[1].parse()?; // Получаем количество запросов из аргументов командной строки
    if n_requests < 1 || n_requests > 100 {
        eprintln!("Number of requests must be between 1 and 100");
        std::process::exit(1); // Выходим из программы с ошибкой
    }

    let mut handles = Vec::with_capacity(n_requests);
    let start_time = Instant::now(); // Фиксируем время старта отправки запросов

    // Создаём асинхронные задачи для отправки запросов
    for _ in 0..n_requests {
        let handle = tokio::spawn(async move {
            let client = Client::new(); // Создаём клиент
            let response = client.get(server_url).send().await?; // Отправляем запрос и ждём ответ
            Ok::<_, reqwest::Error>(response) // Возвращаем результат в виде результата типа Result
        });
        handles.push(handle); // Добавляем задачу в вектор для отслеживания
    }

    let responses = join_all(handles).await; // Ждём завершения всех задач

    let mut min_response_time = Duration::from_secs(u64::MAX); // Инициализируем минимальное время ответа максимальным значением
    let mut max_response_time = Duration::from_secs(0); // Инициализируем максимальное время ответа нулём
    let mut total_response_time = Duration::from_secs(0); // Инициализируем общее время ответов нулём
    let mut n_responses = 0; // Инициализируем счётчик успешных ответов

    // Обрабатываем ответы
    for response in responses {
        match response {
            Ok(_) => {
                // Измеряем время между отправкой запроса и получением ответа
                let response_time = start_time.elapsed();
                if response_time < min_response_time {
                    min_response_time = response_time; // Обновляем минимальное время при необходимости
                }
                if response_time > max_response_time {
                    max_response_time = response_time; // Обновляем максимальное время при необходимости
                }
                total_response_time += response_time; // Обновляем общее время
                n_responses += 1; // Увеличиваем счётчик успешных ответов
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
            }
        }
    }

    let avg_response_time = total_response_time / n_responses as u32; // Вычисляем среднее время ответа

    // Выводим статистику
    println!("Total requests: {}", n_requests);
    println!("Total responses: {}", n_responses);
    println!("Minimum response time: {:?}", min_response_time);
    println!("Maximum response time: {:?}", max_response_time);
    println!("Average response time: {:?}", avg_response_time);

    Ok(()) // Возвращаем успешное завершение программы
}
