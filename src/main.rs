// #![windows_subsystem = "windows"]
use fltk::{
  app::App,
  button::Button,
  frame::Frame,
  group::{experimental::Terminal, Flex},
  input::Input,
  prelude::*,
  window::Window,
};
use hex::FromHex;
use nfd2;
use regex::Regex;
use std::{
  ffi::OsStr,
  fs::{copy, metadata, read_dir, remove_file, OpenOptions},
  io::Read,
  os::windows::fs::FileExt,
  path::{Path, PathBuf},
};

fn main() {
  let l32 = "EB2883FE017518";
  let l64 = "EB2E83FF01751E";
  let b32 = "66B8041090900FB7F885F6750866B804109090";
  let b64 = "66B8041090900FB7D885FF750866B804109090";
  let app = App::default();
  let mut my_window = Window::default()
    .with_size(400, 300)
    .center_screen()
    .with_label("比特彗星解锁");
  let mut col = Flex::default().size_of_parent().column();
  col.set_pad(10);
  let mut row1 = Flex::default().row();
  let label1 = Frame::default().with_label("目标");
  // label1.set_frame(enums::FrameType::BorderBox);
  let mut folder_input = Input::default();
  folder_input.set_value(&find_bitcomet_directory());
  let mut select_folder_btn = Button::default().with_label("📁");
  row1.fixed(&label1, 50);
  row1.fixed(&folder_input, 0);
  row1.fixed(&select_folder_btn, 50);
  row1.end();
  let mut folder_input_cb1 = folder_input.clone(); //路径选择按钮
  select_folder_btn.set_callback(move |_| {
    println!("click");
    if let Ok(res) = nfd2::open_pick_folder(None) {
      match res {
        nfd2::Response::Okay(path) => {
          folder_input_cb1.set_value(path.to_str().unwrap_or(""));
        }
        _ => {}
      }
    }
  });
  let mut start_btn = Button::default().with_label("启动！");
  let folder_input_cb2 = folder_input.clone();
  Frame::default();
  let term = Terminal::default();
  // term.set_text_font(font);
  let mut term1 = term.clone();
  start_btn.set_callback(move |_| {
    let val = folder_input_cb2.value();
    if !val.is_empty() {
      find_and_replace(&mut term1, &val, &l32, &l64, &b32, &b64);
      // dialog::message_default(&exe_str_arr.join("\n"));
    } else {
      term1.append("请先选择 bitcomet 所在文件夹");
    }
  });
  col.fixed(&row1, 50);
  col.fixed(&start_btn, 50);
  col.fixed(&term, 180);
  col.end();
  // my_window.resizable(&row1);
  // my_window.resizable(&term);
  my_window.end();
  my_window.show();
  app.run().unwrap();
}

fn find_bitcomet_directory() -> String {
  // 遍历每个盘符
  for drive_letter in b'C'..=b'Z' {
    let drive_prefix = format!("{}:", drive_letter as char);

    let program_files_path = format!("{}\\Program Files\\BitComet", drive_prefix);
    let program_files_x86_path = format!("{}\\Program Files (x86)\\BitComet", drive_prefix);

    if metadata(&program_files_path).is_ok() {
      return program_files_path;
    } else if metadata(&program_files_x86_path).is_ok() {
      return program_files_x86_path;
    }
  }

  String::from("")
}

fn find_bitcomet_exe(dir_path: &str) -> Vec<PathBuf> {
  let mut arr = Vec::new();
  let reg = Regex::new(r"(?i)^bitcomet.+").unwrap();
  if let Ok(dir) = read_dir(dir_path) {
    for entry in dir {
      if let Ok(entry) = entry {
        if let Some(filename) = entry.file_name().to_str() {
          let a = reg.is_match(filename);
          if a && filename.ends_with(".exe") {
            arr.push(entry.path());
          }
        }
      }
    }
  }
  return arr;
}

fn find_hex_string_in_buffer(buffer: &Vec<u8>, target_buffer: &Vec<u8>) -> Option<usize> {
  if let Some(position) = buffer
    .windows(target_buffer.len())
    .position(|window| window == target_buffer.as_slice())
  {
    println!("Hex string found at offset: {}", position);
    return Some(position);
  } else {
    println!("Hex string not found in the file.");
  }

  None
}

fn modify_language_file(term: &mut Terminal, dir_path_str: &str) {
  let dir_path = Path::new(dir_path_str);
  let zh_cn = dir_path.join("./lang/bitcomet-zh_CN.mo");
  let zh_tw = dir_path.join("./lang/bitcomet-zh_TW.mo");

  if zh_cn.exists() {
    if zh_tw.exists() {
      match remove_file(&zh_tw) {
        Ok(_) => {}
        Err(err) => {
          term.append(&format!("修改语言文件失败 {}\n\n", err.to_string()));
          return;
        }
      }
    }
    match copy(&zh_cn, &zh_tw) {
      Ok(_) => {
        term.append("修改语言文件成功，稍后请在应用中手动切换为繁体中文\n\n");
      }
      Err(err) => {
        term.append(&format!("修改语言文件失败 {}\n\n", err.to_string()));
      }
    }
  } else {
    term.append("语言文件不存在\n\n")
  }
}

fn find_and_replace(
  term: &mut Terminal,
  dir_path_str: &str,
  l32: &str,
  l64: &str,
  b32: &str,
  b64: &str,
) {
  let exe_arr = find_bitcomet_exe(&dir_path_str);
  if exe_arr.len() == 0 {
    term.append("未找到bitcomet可执行文件\n\n")
  }
  for ele in exe_arr {
    let mut exe_buf = Vec::new();
    let file_path = ele.to_str().unwrap_or("");
    let file_name = ele
      .file_name()
      .unwrap_or(OsStr::new(""))
      .to_str()
      .unwrap_or("");
    term.append(&format!("打开文件\n{}\n", file_path));
    match OpenOptions::new().read(true).write(true).open(&ele) {
      Ok(mut file) => {
        match file.read_to_end(&mut exe_buf) {
          Ok(_) => {
            if let Ok(buf_l32) = Vec::from_hex(l32) {
              if let Some(pos) = find_hex_string_in_buffer(&exe_buf, &buf_l32) {
                if let Ok(buf_b32) = Vec::from_hex(b32) {
                  if let Some(_) = find_hex_string_in_buffer(&exe_buf, &buf_b32) {
                    term.append(&format!("32位程序 {} 已解锁，无需修改\n\n", file_name))
                  } else {
                    term.append("开始修改32位程序\n");
                    // exe_buf[pos - 19..pos].clone_from_slice(&buf_b32);
                    term.append("修改成功，开始写入文件\n");
                    match file.seek_write(&buf_b32, (pos - 19) as u64) {
                      Ok(_) => {
                        term.append("写入成功，修改完成\n\n");
                        modify_language_file(term, dir_path_str);
                      }
                      Err(err) => {
                        term.append(&format!("写入失败 {}\n\n", err.to_string()));
                      }
                    }
                  }
                } else {
                  term.append(&format!("{} 16进制字符串转换失败 b32\n\n", b32))
                }
              }
            } else {
              term.append(&format!("{} 16进制字符串转换失败 l32\n\n", l32))
            }
            if let Ok(buf_l64) = Vec::from_hex(l64) {
              if let Some(pos) = find_hex_string_in_buffer(&exe_buf, &buf_l64) {
                if let Ok(buf_b64) = Vec::from_hex(b64) {
                  if let Some(_) = find_hex_string_in_buffer(&exe_buf, &buf_b64) {
                    term.append(&format!("64位程序 {} 已解锁，无需修改\n\n", file_name))
                  } else {
                    term.append("开始修改64位程序\n");
                    exe_buf[pos - 19..pos].clone_from_slice(&buf_b64);
                    term.append("修改成功，开始写入文件\n");
                    match file.seek_write(&buf_b64, (pos - 19) as u64) {
                      Ok(_) => {
                        term.append("写入成功，修改完成\n\n");
                        modify_language_file(term, dir_path_str);
                      }
                      Err(err) => {
                        term.append(&format!("写入失败 {}\n\n", err.to_string()));
                      }
                    }
                  }
                } else {
                  term.append(&format!("{} 16进制字符串转换失败 b64\n\n", b64))
                }
              } else {
                term.append("没有找到可解锁标志\n\n");
              }
            }
          }
          Err(err) => {
            term.append(&format!(
              "读取文件 {} 失败, {}\n",
              file_path,
              err.to_string()
            ));
          }
        };
      }
      Err(err) => {
        term.append(&format!(
          "打开文件 {} 失败, {}\n",
          file_path,
          err.to_string()
        ));
      }
    }
  }
}
