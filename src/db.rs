use self::{data_file_handler::*, defaults::*, modifiers::*};
use crate::id::Id;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use xdg::BaseDirectories;

// TODO: ADD ID TO THE TASKS

type IsoString = String;
type WeekDay = String;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum Status {
    Done,
    Todo,
}

impl Status {
    fn as_boolean(status: Self) -> bool {
        if Status::Done == status {
            true
        } else {
            false
        }
    }

    fn from_boolean(is_done: bool) -> Status {
        if is_done {
            Status::Done
        } else {
            Status::Todo
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TasksDataBase {
    pub tasks: Vec<Task>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub date_created: IsoString,
    pub date_due: IsoString,
    pub status: Status,
    pub repeats: bool,
    pub repeats_every: WeekDay,
    pub id: Id,
}

mod modifiers {
    use super::*;

    pub fn remove_task(task_id: Id) {
        let mut deserialized_tasks_database: TasksDataBase = get_deserialized_tasks_database();
        deserialized_tasks_database
            .tasks
            .remove(search_for_task_index(&deserialized_tasks_database.tasks, task_id).unwrap());
        save_database_changes(deserialized_tasks_database);
    }

    pub fn change_task_name(task_id: Id, name: String) {
        let mut deserialized_tasks_database: TasksDataBase = get_deserialized_tasks_database();
        let task_index: usize =
            search_for_task_index(&deserialized_tasks_database.tasks, task_id).unwrap();
        deserialized_tasks_database.tasks[task_index].name = name;
        save_database_changes(deserialized_tasks_database);
    }

    pub fn toggle_task_status(task_id: Id) {
        let mut deserialized_tasks_database: TasksDataBase = get_deserialized_tasks_database();
        let task_index: usize =
            search_for_task_index(&deserialized_tasks_database.tasks, task_id).unwrap();
        let task_status = deserialized_tasks_database.tasks[task_index].status.clone();
        let inverted_status = Status::from_boolean(!Status::as_boolean(task_status));
        deserialized_tasks_database.tasks[task_index].status = inverted_status;
        save_database_changes(deserialized_tasks_database);
    }

    pub fn change_task_date_due(task_id: Id, new_date: IsoString) {
        let mut deserialized_tasks_database: TasksDataBase = get_deserialized_tasks_database();
        let task_index: usize =
            search_for_task_index(&deserialized_tasks_database.tasks, task_id).unwrap();
        deserialized_tasks_database.tasks[task_index].date_due = new_date;
        save_database_changes(deserialized_tasks_database);
    }

    pub fn toggle_task_repeats(task_id: Id) {
        let mut deserialized_tasks_database: TasksDataBase = get_deserialized_tasks_database();
        let task_index: usize =
            search_for_task_index(&deserialized_tasks_database.tasks, task_id).unwrap();
        let task_repeats: bool = deserialized_tasks_database.tasks[task_index].repeats;
        deserialized_tasks_database.tasks[task_index].repeats = !task_repeats;
        save_database_changes(deserialized_tasks_database);
    }

    pub fn change_task_repeats_every(task_id: Id, new_weekday: WeekDay) {
        let mut deserialized_tasks_database: TasksDataBase = get_deserialized_tasks_database();
        let task_index: usize =
            search_for_task_index(&deserialized_tasks_database.tasks, task_id).unwrap();
        deserialized_tasks_database.tasks[task_index].repeats_every = new_weekday;
        save_database_changes(deserialized_tasks_database);
    }

    pub fn save_database_changes(task_database: TasksDataBase) {
        let reserialized_tasks_database: String = serialize_tasks_database(task_database);
        write_to_data_file(&reserialized_tasks_database);
    }

    pub fn serialize_tasks_database(tasks_database: TasksDataBase) -> String {
        serde_yaml::to_string(&tasks_database).unwrap()
    }
}

mod data_file_handler {
    use super::*;

    pub fn search_for_task_index(tasks: &Vec<Task>, task_id: Id) -> Option<usize> {
        for (index, task) in tasks.iter().enumerate() {
            if task.id == task_id {
                return Some(index);
            }
        }
        None
    }

    pub fn get_tasks_data_file_contents() -> String {
        let mut result: String = String::new();
        create_task_file_if_inexistant();
        fs::File::open(get_tasks_data_file_path())
            .expect("Could not open tasks file")
            .read_to_string(&mut result)
            .expect("Failed reading tasks file");
        result
    }

    pub fn create_task_file_if_inexistant() {
        if !has_tasks_file() {
            create_default_task_file();
        }
    }

    pub fn create_default_task_file() {
        write_to_data_file(&serde_yaml::to_string(&get_default_tasks_database()).unwrap());
    }

    pub fn write_to_data_file(input: &str) {
        fs::File::create(get_tasks_data_file_path())
            .expect("Failed to open tasks data file in write only mode")
            .write_all(input.as_bytes())
            .expect("Could not write to tasks.yaml");
    }

    pub fn append_task_to_data_file(task: Task) {
        let mut tasks_database: TasksDataBase =
            deserialize_tasks_string(&get_tasks_data_file_contents());
        tasks_database.tasks.push(task.clone());
        let serialized_task: String = serde_yaml::to_string(&tasks_database)
            .expect(&format!("Could not serialize task: {:?}", task));
        write_to_data_file(&serialized_task);
    }

    pub fn deserialize_tasks_string(tasks_string_data: &str) -> TasksDataBase {
        serde_yaml::from_str(tasks_string_data).expect("Could not parse tasks.yaml file")
    }

    pub fn get_tasks_data_file_path() -> std::path::PathBuf {
        get_tasks_xdg_directory_path()
            .place_data_file("tasks.yaml")
            .expect("Failed to place data file (~/.local/share/tdm/tasks.yaml)")
    }

    fn get_tasks_xdg_directory_path() -> BaseDirectories {
        BaseDirectories::with_prefix("tdm")
            .expect("Could not create tdm directory (~/.local/share/tdm)")
    }

    pub fn has_tasks_file() -> bool {
        get_tasks_xdg_directory_path()
            .find_data_file("tasks.yaml")
            .is_some()
    }

    pub fn get_deserialized_tasks_database() -> TasksDataBase {
        serde_yaml::from_str(&get_tasks_data_file_contents()).unwrap()
    }
}

mod defaults {
    use super::*;

    pub fn get_default_tasks_database() -> TasksDataBase {
        TasksDataBase {
            tasks: vec![get_default_task()],
        }
    }

    pub fn get_default_task() -> Task {
        Task {
            name: "Welcome!".to_string(),
            description: "You can create new tasks by hitting the key 'n'".to_string(),
            date_due: "2021-08-10".to_string(),
            date_created: "2021-08-10".to_string(),
            status: Status::Todo,
            repeats: false,
            repeats_every: "".to_string(),
            id: 1.0,
        }
    }
}

#[cfg(test)]
/*
 * These tests can only be accurate when
 * using a single thread (cargo test -- --test-threads=1)
 */
mod data_file_handler_tests {
    use super::*;
    use std::process::Command;

    fn cat_task_file_contents() -> String {
        String::from_utf8(
            Command::new("cat")
                .arg(
                    get_tasks_data_file_path()
                        .into_os_string()
                        .into_string()
                        .unwrap(),
                )
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
    }

    fn delete_tasks_data() {
        Command::new("rm")
            .args(["-fr", "~/.local/share/tdm"])
            .spawn()
            .unwrap();
    }

    #[test]
    fn test_append_task_to_file() {
        let test_task: Task = Task {
            name: "Test Task".to_string(),
            description: "Test Task Descritiption".to_string(),
            date_created: "1999-02-13".to_string(),
            date_due: "1999-03-20".to_string(),
            status: Status::Todo,
            repeats: true,
            repeats_every: "monday".to_string(),
            id: crate::id::gen_id(),
        };
        let mut test_tasks_database = get_default_tasks_database();
        create_default_task_file();
        test_tasks_database.tasks.push(test_task.clone());
        append_task_to_data_file(test_task.clone());
        assert_eq!(
            serde_yaml::to_string(&test_tasks_database).unwrap(),
            get_tasks_data_file_contents()
        );
    }

    #[test]
    fn test_create_task_file_if_inexistant() {
        delete_tasks_data();
        create_task_file_if_inexistant();
        assert!(has_tasks_file());
        assert_eq!(
            cat_task_file_contents(),
            serde_yaml::to_string(&get_default_tasks_database()).unwrap()
        )
    }

    #[test]
    fn test_create_default_task_file() {
        create_default_task_file();
        assert_eq!(
            cat_task_file_contents(),
            serde_yaml::to_string(&get_default_tasks_database()).unwrap()
        );
    }

    #[test]
    fn test_get_tasks_data_file_contents() {
        create_default_task_file();
        assert_eq!(
            get_tasks_data_file_contents(),
            serde_yaml::to_string(&get_default_tasks_database()).unwrap()
        );
    }

    #[test]
    fn test_has_task_file() {
        create_default_task_file();
        assert!(has_tasks_file());
    }

    #[test]
    fn test_search_for_task_index() {
        create_default_task_file();
        assert!(search_for_task_index(&get_deserialized_tasks_database().tasks, 1.0).is_some())
    }
}
#[cfg(test)]
mod test_db_modifiers {
    use super::*;
    #[test]
    fn test_remove_task() {
        create_default_task_file();
        remove_task(1.0);
        assert_eq!(
            get_tasks_data_file_contents(),
            "---\ntasks: []\n".to_string()
        );
    }

    #[test]
    fn test_change_task_name() {
        create_default_task_file();
        change_task_name(1.0, "NotWelcome".to_string());
        assert_eq!(
            get_deserialized_tasks_database().tasks[0].name,
            "NotWelcome".to_string()
        )
    }

    #[test]
    fn test_toggle_task_status() {
        create_default_task_file();
        toggle_task_status(1.0);
        assert_eq!(
            get_deserialized_tasks_database().tasks[0].status,
            Status::Done
        )
    }

    #[test]
    fn test_change_task_date_due() {
        let new_date_due: IsoString = "1999-10-19".to_string();
        create_default_task_file();
        change_task_date_due(1.0, new_date_due.clone());
        assert_eq!(
            get_deserialized_tasks_database().tasks[0].date_due,
            new_date_due
        )
    }

    #[test]
    fn test_toggle_task_repeats() {
        create_default_task_file();
        toggle_task_repeats(1.0);
        assert_eq!(get_deserialized_tasks_database().tasks[0].repeats, true)
    }

    #[test]
    fn test_change_task_repeats_every() {
        let expected_day: WeekDay = "Mon".to_string();
        create_default_task_file();
        change_task_repeats_every(1.0, expected_day.clone());
        assert_eq!(
            get_deserialized_tasks_database().tasks[0].repeats_every,
            expected_day
        );
    }
}

#[test]
fn test_deserialize_tasks_string() {
    create_default_task_file();
    let result: TasksDataBase = deserialize_tasks_string(&get_tasks_data_file_contents());
    let welcome_task: Task = result.tasks[0].clone();
    assert_eq!(welcome_task.name, "Welcome!");
    assert_eq!(
        welcome_task.description,
        "You can create new tasks by hitting the key 'n'"
    );
    assert_eq!(welcome_task.date_created, "2021-08-10".to_string());
    assert_eq!(welcome_task.date_due, "2021-08-10".to_string());
    assert_eq!(welcome_task.status, Status::Todo);
    assert_eq!(welcome_task.repeats, false);
    assert_eq!(welcome_task.repeats_every, "".to_string());
}
