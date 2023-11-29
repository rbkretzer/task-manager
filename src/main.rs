mod utils;

use sysinfo::{*, SystemExt};
use dioxus::{ html::*, prelude::* };
use dioxus_desktop::{ Config, WindowBuilder, LogicalSize };
use dioxus_charts::{LineChart, charts::pie::LabelPosition, PieChart};

use futures::StreamExt;
use futures_channel::mpsc::{ unbounded, UnboundedReceiver, UnboundedSender };
use utils::app_props::Network;
use std::cell::Cell;

use crate::utils::{
    sort::{ Sort, SortType },
    app_props::*,
    functions::*
};


fn main() {
    let (sender_procs, receiver_procs) = unbounded();
    let other_procs = sender_procs.clone();

    std::thread::spawn(move || {
        scan_processes(other_procs);
    });

    let (sender_performance, receiver_performance) = unbounded();
    let other_performance = sender_performance.clone();

    std::thread::spawn(move || {
        scan_performance(other_performance);
    });

    dioxus_desktop::launch_with_props(
        app,
        AppProps {
            receiver_procs: Cell::new(Some(receiver_procs)),
            receiver_performance: Cell::new(Some(receiver_performance)),
        },
        Config::default().with_window(
            WindowBuilder::new()
                .with_title("Task manager")
                .with_resizable(true)
                .with_inner_size(LogicalSize::new(800.0, 600.0))
                .with_window_icon(load_icon_by_path("src/img/icons/factory.png"))
        ),
    )
}

fn app(cx: Scope<AppProps>) -> Element {
    let current_tab = use_state(cx, || "tab-processes".to_string());
    let current_graph = use_state(cx, || "CPU".to_string());
    let current_filter = use_state(cx, || "".to_string());
    let options = vec!["CPU".to_string(), "Memória".to_string(), "Network".to_string(), "Discos".to_string()];
    
    let current_processes = use_state(cx, || Vec::new());
    let _ = use_coroutine(cx, |_: UnboundedReceiver<()>| {
        let receiver = cx.props.receiver_procs.take();
        let current_processes = current_processes.to_owned();
        async move {
            if let Some(mut receiver) = receiver {
                while let Some(procs) = receiver.next().await {
                    current_processes.set(procs);
                }
            }
        }
    });
    let sorting_procs = use_state(cx, || Sort{ field: "".to_string(), sort_type: SortType::Unset });
    let sort_procs = sorting_procs.get();
    let procs = current_processes.get().to_vec();
    let mut sorted_procs = procs.into_iter().filter(|proc| proc.name.contains(current_filter.as_str()) || proc.pid.to_string().contains(current_filter.as_str())).collect::<Vec<_>>();
    if sort_procs.sort_type.eq(&SortType::Asc) {
        sorted_procs.sort_by(|p1, p2| MyProcess::new(p1).compare(MyProcess::new(p2), sort_procs.field.as_str()));
    } else if sort_procs.sort_type.eq(&SortType::Desc) {
        sorted_procs.sort_by(|p1, p2| MyProcess::new(p2).compare(MyProcess::new(p1), sort_procs.field.as_str()));
    }

    let current_performance = use_state(cx, || Performance::default());
    let _ = use_coroutine(cx, |_: UnboundedReceiver<()>| {
        let receiver = cx.props.receiver_performance.take();
        let current_performance = current_performance.to_owned();
        async move {
            if let Some(mut receiver) = receiver {
                while let Some(performace) = receiver.next().await {
                    current_performance.set(performace);
                }
            }
        }
    });
    let performance = current_performance.get();
    
    cx.render(rsx!(
        style { include_str!("./assets/styles.css") },
        style { include_str!("./assets/options.css") },
        link { rel: "stylesheet", href:"https://fonts.googleapis.com/icon?family=Material+Icons" },
        body {
            section { class: "tabs wrapper",
                input { name: "tab-processes", id: "tab-processes", r#type: "radio", class: "tabs-radio",
                    checked: is_tab(current_tab, "tab-processes") ,
                    onclick: move |_| set_tab(current_tab, "tab-processes"),
                },
                label { r#for: "tab-processes", class: "tabs-label", "Processos" },
                table { class: "tabs-content",
                    thead {
                        tr {
                            th { onclick: move |_| set_sorting(sorting_procs, "name"), "Nome", sort_procs.clone().sorting(cx, "name".to_string())},
                            th { onclick: move |_| set_sorting(sorting_procs, "pid"), "PID", sort_procs.clone().sorting(cx, "pid".to_string())},
                            th { onclick: move |_| set_sorting(sorting_procs, "cpu_usage"), "CPU", sort_procs.clone().sorting(cx, "cpu_usage".to_string())},
                            th { onclick: move |_| set_sorting(sorting_procs, "memory"), "Memória", sort_procs.clone().sorting(cx,"memory".to_string())},
                        },
                    },
                    tbody {
                        for proc in sorted_procs {
                            rsx!(
                                tr {
                                    td { "{proc.name}" },
                                    td { "{proc.pid}" },
                                    td { "{proc.cpu_usage}" },
                                    td { "{proc.memory}" },
                                },
                            )
                        }
                    }
                },

                input { name: "tab-performance", id: "tab-performance", r#type: "radio", class: "tabs-radio",
                    checked: is_tab(current_tab, "tab-performance"),
                    onclick: move |_| set_tab(current_tab, "tab-performance"),
                },
                label { r#for: "tab-performance", class: "tabs-label", "Desempenho" },
                div { class: "tabs-content",
                    select {  onchange: move |evt| current_graph.set(evt.data.value.clone()),
                        for opt in options.clone() {
                            rsx!(
                                option { label: "{opt}", value : "{opt}"}
                            )
                        }
                    },
                    match current_graph.get().as_str() {
                        "CPU" => rsx!(div {
                            if !performance.cpus.is_empty() {
                                rsx!(LineChart {
                                    width: "100%",
                                    height: "100%",
                                    padding_top: 30,
                                    padding_left: 50,
                                    padding_right: 90,
                                    padding_bottom: 30,
                                    show_grid_ticks: true,
                                    show_dotted_grid: false,
                                    series: match performance.cpus.is_empty() {
                                        false => performance.cpus.iter().map(|c| c.uses.clone()).collect::<Vec<_>>(),
                                        true => Vec::new()
                                    },
                                    labels: vec!["60".to_string(), "0".to_string()],
                                    series_labels: match performance.cpus.is_empty() {
                                        false => performance.cpus.iter().map(|c| c.name.clone()).collect::<Vec<_>>(),
                                        true => Vec::new(),
                                    },
                                })
                            } else {
                                rsx!("Carregando...")
                            }
                        }),
                        "Memória" => rsx!(div { 
                            div { style: "width: 100%; overflow: hidden;",
                                div { style: "width: 50%; float: left; ",
                                    "Memória"
                                    div { format! { "Total: {:.02} GiB", performance.mem.total} },
                                    div { format! { "Em uso: {:.02} GiB", performance.mem.used} },
                                }
                                div { style: "margin-left: 15px; ",
                                    PieChart {
                                        width: "50%",
                                        height: "100%",
                                        start_angle: 50.0,
                                        label_position: LabelPosition::Outside,
                                        label_offset: 27.0,
                                        donut: true,
                                        padding: 20.0,
                                        series: vec![performance.mem.used as f32, performance.mem.free as f32],
                                        labels: vec!["Em uso".to_string(), "Livre".to_string()],
                                    }
                                }
                            },
                            div { style: "width: 100%; overflow: hidden;",
                                div { style: "width: 50%; float: left; ",
                                    "Swap",
                                    div { format! { "Total: {:.02} GiB", performance.swap.total} },
                                    div { format! { "Em uso: {:.02} GiB", performance.swap.used} },
                                }
                                PieChart {
                                    width: "50%",
                                    height: "100%",
                                    start_angle: 50.0,
                                    label_position: LabelPosition::Outside,
                                    label_offset: 27.0,
                                    donut: true,
                                    padding: 20.0,
                                    series: vec![performance.swap.used as f32, performance.swap.free as f32],
                                    labels: vec!["Em uso".to_string(), "Livre".to_string()],
                                }
                            },
                        }),
                        "Network" => rsx!(
                            for net in performance.networks.to_vec() {
                                rsx! {
                                    div { style: "text-align: center; padding-bottom: 15px;",
                                        div { net.name },
                                        div { format! { "Transmitido: {:?} B - Recebido: {:?} B", net.transmitted, net.received } },
                                        div { format! { "Total transmitido: {:?} B - Total recebido: {:?} B", net.total_transmitted, net.total_received } },
                                    }
                                }
                            }
                        ),
                        "Discos" => rsx!(
                            for disk in performance.disks.to_vec() {
                                div { style: "width: 100%; overflow: hidden;",
                                    div { style: "width: 50%; float: left; text-align: center; padding-bottom: 15px;",
                                        div { disk.local },
                                        div { format! { "{}: {} - Espaço: {:?} GiB", disk.kind, disk.structure, disk.space } },
                                        div { format! { "Removível: {}", match disk.removable { true => "Sim", false => "Não" }} },
                                    }
                                    PieChart {
                                        width: "50%",
                                        height: "100%",
                                        start_angle: 50.0,
                                        label_position: LabelPosition::Outside,
                                        label_offset: 27.0,
                                        donut: true,
                                        padding: 20.0,
                                        series: vec![disk.used as f32, disk.free as f32],
                                        labels: vec!["Em uso".to_string(), "Livre".to_string()],
                                    }
                                }
                            }
                        ),
                        _ => rsx!(div { "Deu merda" }),
                    }
                },
                
                input { class: "search", name: "filter", id: "filter", disabled: is_tab(current_tab, "tab-performance"),
                    placeholder: "Nome ou PID",
                    oninput: move |evt| current_filter.set(evt.value.clone()),
                }
            }
        }
    ))
}

fn set_tab(tab: &UseState<String>, value: &str) {
    tab.set(value.to_string());
}

fn is_tab(tab: &UseState<String>, value: &str) -> bool {
    return tab.get().eq(value);
}

fn set_sorting(sorting: &UseState<Sort>, field_to_sort: &str) {
    if sorting.get().field.ne(field_to_sort) {
        sorting.set(Sort {field: field_to_sort.to_string(), sort_type: SortType::Desc});
        return;
    }
    let sort_type = match sorting.get().sort_type {
        SortType::Desc => SortType::Asc,
        SortType::Asc => SortType::Unset,
        SortType::Unset => SortType::Desc,
    };
    sorting.set(Sort {field: field_to_sort.to_string(), sort_type});
}

fn scan_processes(sender: UnboundedSender<Vec<MyProcess>>) {
    loop {
        let mut procs: Vec<MyProcess> = Vec::new();
        let mut sys_info = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::everything()));
        sys_info.refresh_cpu();
        for process in sys_info.processes() {
            
            let proc = MyProcess {
                pid: process.1.pid().as_u32(),
                name: process.1.name().to_string(),
                memory: process.1.memory() / 1000000,
                cpu_usage: process.1.cpu_usage(),
                read_bytes: process.1.disk_usage().read_bytes,
                written_bytes: process.1.disk_usage().written_bytes,
            };
            procs.push(proc);
        }
        let _ = sender.unbounded_send(procs);
        std::thread::sleep(std::time::Duration::from_secs(1))
    }
}

fn scan_performance(sender: UnboundedSender<Performance>) {
    // let mut current_cpus: Vec<MyCpu> = Vec::new();
    loop {
        let mut sys_info = System::new_with_specifics(RefreshKind::everything().without_processes());
        sys_info.refresh_all();

        let mut disks = sys_info.disks().iter().map(|d| d.clone()).collect::<Vec<_>>();
        disks.sort_by(|d1, d2| d1.mount_point().as_os_str().cmp(d2.mount_point().as_os_str()));
        let mut struct_disks = Vec::new();
        for disk in disks {
            struct_disks.push(MyDisk {
                local: disk.mount_point().as_os_str().to_str().unwrap().to_string(),
                space: disk.total_space() / 1000000000,
                kind: format! { "{:?}", disk.kind() },
                structure: String::from_utf8(disk.file_system().to_vec()).unwrap(),
                removable: disk.is_removable(),
                used: disk.total_space() - disk.available_space(),
                free: disk.available_space(),
            });
        }

        let mut networks = sys_info.networks().iter().collect::<Vec<_>>();
        networks.sort_by(|n1, n2| n1.0.cmp(n2.0));
        let mut struct_networks = Vec::new();
        for (interface_name, network) in networks {
            struct_networks.push(Network {
                name: interface_name.clone(),
                transmitted: network.transmitted(),
                received: network.received(),
                total_transmitted: network.total_transmitted(),
                total_received: network.total_received(),
            });
        }

        // let other_cpus = current_cpus.to_vec();
        let mut new_cpus = Vec::new();
        for cpu in sys_info.cpus() {
            // let founded = other_cpus.to_vec().into_iter().find(|c| c.name.eq(cpu.name()));
            // if let Some(mut founded) = founded {
            //     if founded.uses.len().eq(&60) {
            //         founded.uses.remove(0);
            //     }

            //     founded.uses.push(cpu.cpu_usage());
            //     current_cpus.push(founded);
            // } else {
                new_cpus.push(MyCpu {
                    name: cpu.name().to_string(),
                    uses: vec![cpu.cpu_usage()],
                });
            // }
        }
        // for idx in 0..sys_info.cpus().len() - 1 {
        //     current_cpus.remove(idx);
        // }
        let _ = sender.unbounded_send(Performance {
            cpus: new_cpus,
            mem: Mem {
                total: sys_info.total_memory() / 1000000000,
                used: sys_info.used_memory() / 1000000000,
                free: (sys_info.total_memory() - sys_info.used_memory()) / 1000000000,
            },
            swap: Swap {
                total: sys_info.total_swap() / 1000000000,
                used: sys_info.used_swap() / 1000000000,
                free: (sys_info.total_swap() - sys_info.used_swap()) / 1000000000,
            },
            networks: struct_networks,
            disks: struct_disks,
        });
        print!("passei aqui");
        std::thread::sleep(std::time::Duration::from_secs(1))
    }
} 