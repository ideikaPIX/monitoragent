use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworksExt, NetworkExt};
use std::thread::sleep;
use std::time::Duration;
use crossterm::{ExecutableCommand, cursor, terminal::{Clear, ClearType}};
use std::io::{stdout, Write};
use termion::color;

fn main() {
    let mut system = System::new_all();

    // Инициализация для хранения предыдущих значений
    let mut old_network_data = system.networks().iter().map(|(name, data)| {
        (name.clone(), (data.received(), data.transmitted()))
    }).collect::<Vec<_>>();

    // Инициализация stdout заранее
    let mut stdout = stdout();

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
            let used_disk_space = (total_disk_space - disk.available_space() as f64) / total_disk_space * 100.0;
            let used_gb = (total_disk_space - disk.available_space() as f64) / 1_073_741_824.0;
            let total_gb = total_disk_space / 1_073_741_824.0;
            disk_info.push((used_disk_space, used_gb, total_gb));
        }

        // Получение сетевой активности
        let new_network_data = system.networks().iter().map(|(name, data)| {
            (name.clone(), (data.received(), data.transmitted()))
        }).collect::<Vec<_>>();

        let mut received_speed = 0;
        let mut transmitted_speed = 0;

        for (old, new) in old_network_data.iter().zip(new_network_data.iter()) {
            let (old_received, old_transmitted) = old.1;
            let (new_received, new_transmitted) = new.1;

            // Проверка, чтобы избежать переполнения
            if new_received >= old_received {
                received_speed += new_received - old_received;
            }
            if new_transmitted >= old_transmitted {
                transmitted_speed += new_transmitted - old_transmitted;
            }
        }

        // Обновление старых значений сетевой активности
        old_network_data = new_network_data;

        // Конвертация байтов в КБ/с
        let received_speed_kbps = (received_speed as f64) / 1024.0;
        let transmitted_speed_kbps = (transmitted_speed as f64) / 1024.0;

        // Функция для создания строки прогресса с цветом
        fn create_progress_bar(usage: f64) -> String {
            let filled = (usage / 10.0).round() as usize;
            let empty = 10 - filled;
            let mut bar = String::new();
        
            // Green color for the first 50%
            if usage <= 50.0 {
                bar = format!("\x1b[32m{}\x1b[0m", "#".repeat(filled));
            }
            // Yellow color for the next 25%
            else if usage <= 75.0 {
                let green_part = 5;
                let yellow_part = filled - green_part;
                bar = format!(
                    "\x1b[32m{}\x1b[33m{}\x1b[0m",
                    "#".repeat(green_part),
                    "#".repeat(yellow_part)
                );
            }
            // Red color for the remaining 25%
            else {
                let green_part = 5;
                let yellow_part = 2;
                let red_part = filled - green_part - yellow_part;
                bar = format!(
                    "\x1b[32m{}\x1b[33m{}\x1b[38;5;208m{}\x1b[0m",
                    "#".repeat(green_part),
                    "#".repeat(yellow_part),
                    "#".repeat(red_part)
                );
            }
        
            format!("[{}{}]", bar, " ".repeat(empty))
        }

        // Очистка экрана и установка курсора в начало
        stdout.execute(Clear(ClearType::All)).unwrap();
        stdout.execute(cursor::MoveTo(0, 0)).unwrap();

        // Вывод данных CPU и RAM
        println!("CPU {}", create_progress_bar(cpu_usage as f64));
        println!("RAM {}", create_progress_bar(ram_usage));

        // Вывод сетевой активности
        println!("Network Received: {:.2} KB/s", received_speed_kbps);
        println!("Network Transmitted: {:.2} KB/s", transmitted_speed_kbps);

        // Разделительная черта
        println!("{}", "-".repeat(50));

        // Вывод данных по каждому диску справа
        for (i, (usage, used, total)) in disk_info.iter().enumerate() {
            let progress_bar = create_progress_bar(*usage);
            let disk_info_str = format!("ROM {} (Disk {}: {:.2} GB / {:.2} GB)", progress_bar, i + 1, used, total);
            if *usage > 70.0 {
                println!("{:<80}\x1b[48;5;208m !!! overloaded \x1b[0m", disk_info_str);
            } else {
                println!("{:<80}", disk_info_str);
            }
        }

        // Пауза перед следующим обновлением
        sleep(Duration::from_secs(1));
    }
}
