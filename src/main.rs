use anyhow::Result;
use std::env;
use std::io::BufReader;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process;
use std::vec;
use std::{fs, io};
use tokio_stream::StreamExt;
use tonic::transport::Channel;
use urchin::command_server;
use urchin::output;
use urchin::root;
use urchin::version::BazelVersion;

#[tokio::main]
async fn main() -> Result<()> {
    let cwd = env::current_dir().expect("could not determine working directory");
    let root = root::get_root(&cwd).unwrap_or(fs::canonicalize("./examples/workspace").unwrap());

    let version = BazelVersion::from_root(&root).expect("could not determine Bazel version");

    let output_user_root = output::get_default_user_output_root();
    let output_base = output::get_default_output_base(&output_user_root, &root);
    let install = output_base.join("install");
    let server = output_base.join("server");

    if !install.exists() {
        println!("Extracting Bazel installation...");
        let file = fs::File::open(version.get()).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            if let Some(name) = file.enclosed_name() {
                let dest = install.join(name);
                let parent = dest.parent().unwrap();
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap()
                }
                let mode = file.unix_mode().expect("corrupted archive?");
                let mut out = fs::File::create(dest).unwrap();
                out.set_permissions(fs::Permissions::from_mode(mode))
                    .expect("failed to set mode during extraction");
                io::copy(&mut file, &mut out).unwrap();
            }
        }
    }

    println!("{}", output_base.to_string_lossy());
    // Create output_base/server directory and write the server.pid.txt file
    fs::create_dir_all(&server).unwrap();
    let pid_path = server.join("server.pid.txt");
    if !pid_path.exists() {
        fs::File::create(&pid_path)
        .unwrap()
        .write_all(process::id().to_string().as_bytes())
        .unwrap();

    // let mut server_proc = process::Command::new(install.join("embedded_tools/jdk/bin/java"));
    // let mut server_proc = process::Command::new("/Library/Java/JavaVirtualMachines/temurin-17.jdk/Contents/Home/bin/java");
    let mut server_proc = process::Command::new("/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home/bin/java");
    // let mut server_proc = process::Command::new("/Users/thesayyn/Downloads/zulu23.32.11-ca-jdk23.0.2-macosx_aarch64/zulu-23.jdk/Contents/Home/bin/java");
    // Redirect stderr into java_log.
    let java_log = fs::File::create(output_base.join("java.log")).unwrap();
    server_proc.stderr(process::Stdio::from(java_log));
    server_proc.env("DYLD_INSERT_LIBRARIES", "/Users/thesayyn/Documents/urchin/spine/target/debug/libspine.dylib");
    // server_proc.env("RUST_BACKTRACE", "1");
    server_proc
        // Jvm arguments
        .arg("--add-opens=java.base/java.lang=ALL-UNNAMED")
        .arg("-Xverify:none")
        // .arg("-Xlog:all=info:stderr:uptime,level,tags")
        .arg("-Dfile.encoding=ISO-8859-1")
        .arg("-Duser.country=")
        .arg("-Duser.language=")
        .arg("-Duser.variant=")
        .arg("-jar")
        .arg(install.join("A-server.jar"))
        // Bazel arguments
        .arg(format!(
            "--output_user_root={}",
            output_user_root.to_string_lossy()
        ))
        .arg(format!("--output_base={}", output_base.to_string_lossy()))
        .arg(format!("--install_base={}", install.to_string_lossy()))
        .arg(format!(
            "--failure_detail_out={}",
            output_base
                .join("failure_detail.rawproto")
                .to_string_lossy()
        ))
        .arg(format!("--workspace_directory={}", &root.to_string_lossy()));

        let mut child = server_proc.spawn()?;

        let pid_path = pid_path.clone();
        // TODO: always set handler and kill by pid
        ctrlc::set_handler(move || {
            let _ = fs::remove_file(&pid_path);
            child.kill().unwrap();
            println!("Killing server.");
            process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");

        loop {
            if server.join("request_cookie").try_exists().unwrap_or(false) {
                break;
            }
        }    
    }
    
        
    println!("Server pid is {}", fs::read_to_string(&pid_path).unwrap());

    let command_port = fs::read_to_string(server.join("command_port")).unwrap();

    let channel = Channel::from_shared(format!("http://{}", command_port))
        .unwrap()
        .connect()
        .await?;

    let mut cmd =
        command_server::command_server::command_server_client::CommandServerClient::new(channel);

    let req = command_server::command_server::RunRequest {
        client_description: String::from("urchin"),
        cookie: fs::read_to_string(server.join("request_cookie")).unwrap(),
        arg: env::args()
            .skip(1)
            .chain(vec!["--isatty".to_string()])
            .map(|f| f.as_bytes().to_vec())
            .collect(),
        block_for_lock: false,
        preemptible: false,
        command_extensions: vec![],
        invocation_policy: String::new(),
        startup_options: vec![],
    };

    let mut resp = cmd.run(tonic::Request::new(req)).await?.into_inner();

    while let Some(recv) = resp.next().await {
        if let Ok(recv) = recv {
            print!("{}", String::from_utf8(recv.standard_error).unwrap());
            print!("{}", String::from_utf8(recv.standard_output).unwrap());
        } else {
            println!("Bazel server crashed, printing the log.");
            let log =
                fs::File::open(output_base.join("java.log")).expect("failed to open java.log");

            io::copy(&mut BufReader::new(log), &mut io::stderr().lock()).unwrap();
        }
    }

    Ok(())
}
