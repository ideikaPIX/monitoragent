use sysinfo::{CpuExt, DiskExt, NetworkExt, NetworksExt, System, SystemExt};
use std::{
    fs::File,
    io::{Read, Write},
    time::Duration,
};
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    terminal::{Clear, ClearType}, ExecutableCommand,
};
use std::io::{stdin, stdout};
use chrono::Local;

// Структура для хранения настроек приложения
struct AppSettings {
    pub show_progress_bars: bool,
    pub show_debug_info: bool,
}

impl AppSettings {
    // Конструктор для создания настроек по умолчанию
    fn new() -> AppSettings {
        AppSettings {
            show_progress_bars: true,
            show_debug_info: false,
        }
    }

    // Сохранение настроек в файл
    fn save_settings(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        writeln!(file, "{}", if self.show_progress_bars { "bars" } else { "percent" })?;
        writeln!(file, "{}", if self.show_debug_info { "Enable" } else { "Disable" })?;
        Ok(())
    }

    // Загрузка настроек из файла
    fn load_settings(filename: &str) -> std::io::Result<AppSettings> {
        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut lines = contents.lines();
        let show_progress_bars = lines.next().unwrap_or("") == "bars";
        let show_debug_info = lines.next().unwrap_or("") == "Enable";

        Ok(AppSettings {
            show_progress_bars,
            show_debug_info,
        })
    }
}

fn main() {
    let mut system = System::new_all();

    // Инициализация для хранения предыдущих значений сетевой активности
    let mut old_network_data = system
        .networks()
        .iter()
        .map(|(name, data)| (name.clone(), (data.received(), data.transmitted())))
        .collect::<Vec<_>>();

    // Инициализация stdout заранее
    let mut stdout = stdout();

    // Настройки приложения
    let mut settings = match AppSettings::load_settings("settings.txt") {
        Ok(settings) => settings,
        Err(_) => AppSettings::new(),
    };

    loop {
        // Очистка экрана и установка курсора в начало
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        // Вывод главного меню
        println!("Main Menu:");
        println!("1. Start agent");
        println!("2. Settings");
        println!("3. Credits");
        println!("4. Debug Log");
        println!("5. Exit");

        // Получение выбора пользователя
        println!("\nEnter your choice:");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");

        // Обработка выбора пользователя
        match input.trim() {
            "1" => {
                println!("Starting agent...");
                run_agent(&mut system, &mut stdout, &mut old_network_data, &settings);
            }
            "2" => {
                show_settings_menu(&mut settings);
                // Сохранение настроек после изменения
                settings.save_settings("settings.txt").unwrap();
            }
            "3" => {
                println!("Credits:");
                println!("GitHub: https://github.com/ideikiPIX");
                println!("Author: ideikiPIX");
                println!("\nPress Enter to return to the main menu...");
                let mut return_input = String::new();
                stdin().read_line(&mut return_input).expect("Failed to read line");
            }
            "4" => {
                println!("Generating debug log...");
                generate_debug_log(&mut system, &settings);
            }
            "5" => {
                println!("Exiting...");
                // Сохранение настроек перед выходом
                settings.save_settings("settings.txt").unwrap();
                break;
            }
            _ => {
                println!("Invalid choice. Please enter a number from 1 to 5.");
            }
        }
    }
}

// Функция для запуска агента
fn run_agent(
    system: &mut System,
    stdout: &mut std::io::Stdout,
    old_network_data: &mut Vec<(String, (u64, u64))>,
    settings: &AppSettings,
) {
    loop {
        system.refresh_all();

        // Получение загрузки CPU
        let cpu_usage = system.global_cpu_info().cpu_usage();

        // Получение использования RAM
        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        let ram_usage = (used_memory / total_memory) * 100.0;

        // Получение использования дисков (по каждому диску отдельно)
        let mut disk_info = vec![];
        for disk in system.disks() {
            let total_disk_space = disk.total_space() as f64;
            let used_disk_space = (total_disk_space - disk.available_space() as f64)
                / total_disk_space
                * 100.0;
            let used_gb = (total_disk_space - disk.available_space() as f64) / 1_073_741_824.0;
            let total_gb = total_disk_space / 1_073_741_824.0;
            disk_info.push((used_disk_space, used_gb, total_gb));
        }

        // Получение сетевой активности
        let new_network_data = system
            .networks()
            .iter()
            .map(|(name, data)| (name.clone(), (data.received(), data.transmitted())))
            .collect::<Vec<_>>();

        let mut received_speed = 0;
        let mut transmitted_speed = 0;

        for (
            (_, (old_received, old_transmitted)),
            (_, (new_received, new_transmitted)),
        ) in old_network_data.iter().zip(new_network_data.iter())
        {
            // Проверка, чтобы избежать переполнения
            if new_received >= old_received {
                received_speed += new_received - old_received;
            }
            if new_transmitted >= old_transmitted {
                transmitted_speed += new_transmitted - old_transmitted;
            }
        }

        // Обновление старых значений сетевой активности
        *old_network_data = new_network_data;

        // Конвертация байтов в КБ/с
        let received_speed_kbps = (received_speed as f64) / 1024.0;
        let transmitted_speed_kbps = (transmitted_speed as f64) / 1024.0;

        // Очистка экрана и установка курсора в начало
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        // Вывод данных CPU и RAM в зависимости от настроек
        if settings.show_progress_bars {
            println!("CPU {}", create_progress_bar(cpu_usage as f64));
            println!("RAM {}", create_progress_bar(ram_usage));
        } else {
            println!("CPU {}", create_load_display(cpu_usage as f64));
            println!("RAM {}", create_load_display(ram_usage));
        }

        // Вывод сетевой активности
        println!("Network Received: {:.2} KB/s", received_speed_kbps);
        println!("Network Transmitted: {:.2} KB/s", transmitted_speed_kbps);

        // Разделительная черта
        println!("{}", "-".repeat(50));

        // Вывод данных по каждому диску
        for (i, (usage, used, total)) in disk_info.iter().enumerate() {
            let disk_load = calculate_disk_load(*usage, settings);
            let disk_info_str = format!(
                "ROM {} (Disk {}: {:.2} GB / {:.2} GB)",
                disk_load,
                i + 1,
                used,
                total
            );
            if *usage > 70.0 {
                println!(
                    "{:<80}\x1b[48;5;208m !!! overloaded \x1b[0m",
                    disk_info_str
                );
            } else {
                println!("{:<80}", disk_info_str);
            }
        }

        println!("\nPress 'q' and enter to return to the main menu.");

        // Проверка нажатия клавиши 'q' для выхода из агента
        if crossterm::event::poll(Duration::from_secs(1)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                if event.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }
}

// Функция для создания строки прогресса с цветом
fn create_progress_bar(usage: f64) -> String {
    let filled = (usage / 10.0).round() as usize;
    let empty = 10 - filled;
    let bar = match usage {
        usage if usage <= 50.0 => format!("\x1b[32m{}\x1b[0m", "#".repeat(filled)),
        usage if usage <= 75.0 => {
            let green_part = 5;
            let yellow_part = filled - green_part;
            format!(
                "\x1b[32m{}\x1b[33m{}\x1b[0m",
                "#".repeat(green_part),
                "#".repeat(yellow_part)
            )
        }
        _ => {
            let green_part = 5;
            let yellow_part = 2;
            let red_part = filled - green_part - yellow_part;
            format!(
                "\x1b[32m{}\x1b[33m{}\x1b[38;5;208m{}\x1b[0m",
                "#".repeat(green_part),
                "#".repeat(yellow_part),
                "#".repeat(red_part)
            )
        }
    };

    format!("[{}{}]", bar, " ".repeat(empty))
}

// Функция для создания строки прогресса в процентах с цветом
fn create_load_display(usage: f64) -> String {
    let usage_percent = usage.round() as usize;
    match usage {
        usage if usage <= 50.0 => format!("\x1b[32m{}%\x1b[0m", usage_percent),
        usage if usage <= 75.0 => format!("\x1b[33m{}%\x1b[0m", usage_percent),
        _ => format!("\x1b[38;5;208m{}%\x1b[0m", usage_percent),
    }
}

// Новая функция для окрашивания процентных значений загрузки дисков
fn color_disk_percentage(usage: f64) -> String {
    let usage_percent = usage.round() as usize;
    match usage {
        usage if usage <= 50.0 => format!("\x1b[32m{}%\x1b[0m", usage_percent),
        usage if usage <= 75.0 => format!("\x1b[33m{}%\x1b[0m", usage_percent),
        _ => format!("\x1b[31m{}%\x1b[0m", usage_percent),
    }
}

// Функция для вычисления загрузки диска и возвращения строки прогресса или процентов в зависимости от настроек
fn calculate_disk_load(usage: f64, settings: &AppSettings) -> String {
    if settings.show_progress_bars {
        create_progress_bar(usage)
    } else {
        color_disk_percentage(usage)
    }
}

// Функция для отображения меню настроек
fn show_settings_menu(settings: &mut AppSettings) {
    loop {
        // Очистка экрана и установка курсора в начало
        print_main_menu_header();

        // Отображение текущих настроек
        let load_display_type = if settings.show_progress_bars {
            "\x1b[32mbars\x1b[0m"
        } else {
            "\x1b[32mpercent\x1b[0m"
        };

        let debug_info_toggle = if settings.show_debug_info {
            "\x1b[32mEnable\x1b[0m"
        } else {
            "\x1b[31mDisable\x1b[0m"
        };

        println!("Settings:");
        println!("1. Change the load display type (Currently: {})", load_display_type);
        println!("2. Debug information toggled (Currently: {})", debug_info_toggle);
        println!("3. Back to Main Menu");

        // Получение выбора пользователя
        println!("\nEnter your choice:");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");

        // Обработка выбора пользователя
        match input.trim() {
            "1" => {
                settings.show_progress_bars = !settings.show_progress_bars;
                println!("Load display type now: {}", if settings.show_progress_bars { "bars" } else { "percent" });
            }
            "2" => {
                settings.show_debug_info = !settings.show_debug_info;
                println!("Debug information toggled: {}", if settings.show_debug_info { "Enabled" } else { "Disabled" });
            }
            "3" => {
                break;
            }
            _ => {
                println!("Invalid choice. Please enter a number from 1 to 3.");
            }
        }
    }
}

// Функция для генерации отладочного лога
fn generate_debug_log(system: &mut System, settings: &AppSettings) {
    let mut debug_log_file = File::create("debug_log.txt").expect("Failed to create debug log file");

    // Запись средней загрузки CPU и RAM за все время использования
    let average_cpu_usage = system.global_cpu_info().cpu_usage();
    let total_memory = system.total_memory() as f64;
    let used_memory = system.used_memory() as f64;
    let average_ram_usage = (used_memory / total_memory) * 100.0;
    writeln!(debug_log_file, "Average CPU Usage: {:.2}%", average_cpu_usage).unwrap();
    writeln!(debug_log_file, "Average RAM Usage: {:.2}%", average_ram_usage).unwrap();

    // Запись скачанного и загруженного сетевого трафика
    let total_network_received = system
        .networks()
        .iter()
        .map(|(_, data)| data.received())
        .sum::<u64>() as f64
        / 1_073_741_824.0; // переводим в гигабайты
    let total_network_transmitted = system
        .networks()
        .iter()
        .map(|(_, data)| data.transmitted())
        .sum::<u64>() as f64
        / 1_073_741_824.0; // переводим в гигабайты
    writeln!(debug_log_file, "Total Network Received: {:.2} GB", total_network_received).unwrap();
    writeln!(debug_log_file, "Total Network Transmitted: {:.2} GB", total_network_transmitted).unwrap();

    // Запись использования каждого диска
    for (i, disk) in system.disks().iter().enumerate() {
        let total_disk_space = disk.total_space() as f64;
        let used_disk_space = (total_disk_space - disk.available_space() as f64) / total_disk_space * 100.0;
        writeln!(debug_log_file, "Disk {} Usage: {:.2}%", i + 1, used_disk_space).unwrap();
    }

    // Вывод сохраненной информации из настроек, если включено отображение отладочной информации
    if settings.show_debug_info {
        let current_time = Local::now();
        let formatted_time = current_time.format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(debug_log_file, "Timestamp: {}", formatted_time).unwrap();
    }

    

    // Закрытие файла
    drop(debug_log_file);

    // Чтение и вывод содержимого debug_log.txt в консоль
    let mut debug_log_content = String::new();
    File::open("debug_log.txt")
        .and_then(|mut file| file.read_to_string(&mut debug_log_content))
        .expect("Failed to read debug log file");

    println!("Debug Log:\n{}", debug_log_content);

    // Ожидание нажатия Enter для очистки консоли
    println!("Press Enter to return to the main menu...");
    let mut return_input = String::new();
    stdin().read_line(&mut return_input).expect("Failed to read line");
    
    
    println!("Debug log generated successfully.");
}




// Вспомогательная функция для вывода заголовка главного меню
fn print_main_menu_header() {
    // Очистка экрана и установка курсора в начало
    let mut stdout = stdout();
    stdout.execute(Clear(ClearType::All)).unwrap();
    stdout.execute(cursor::MoveTo(0, 0)).unwrap();
}
