use super::marks::{ MarksRecords, StudentId, Marks };

use std::fmt::Display;
use std::cell::RefCell;
use std::rc::Rc;

use stdweb::web::
    { Element, HtmlElement, IElement, IHtmlElement
    , IEventTarget, IParentNode
    , document, event, window
    };
use stdweb::unstable::TryInto;
use stdweb::web::html_element::{InputElement, TextAreaElement};
use stdweb::web::event::IKeyboardEvent;

// Should only be called once per DOM.
pub fn run() {
    let web_ui = init_on_dom();
    load_event_handlers(web_ui);
}

// html element ids
const TAB_STEP_1: &str = "#step_1";
const TAB_STEP_2: &str = "#step_2";
const STUDENTS  : &str = "#students";
const DONE_BTN  : &str = "#done_btn";
const INPUT     : &str = "#input";
const OUTPUT    : &str = "#output";

// All the elements and data of the UI.
struct WebUI {
    step_1: HtmlElement,
    step_2: HtmlElement,
    students: TextAreaElement,
    done_btn: InputElement,
    input: InputElement,
    output: TextAreaElement,
    marks_records: Rc<RefCell<MarksRecords>>
}

// Supported inputs.
enum Input {
    Query (StudentId),
    MarksRecord (Marks),
    Export,
    Clear
}
use self::Input::*;

// Initialize WebUI by loading event handlers to the preset elements on DOM.
// Data is loaded from local storage if available.
fn init_on_dom() -> Rc<WebUI> {
    let msg = |t, s| format!("failed unwrapping {} {}", t, s);
    let step_1 = unwrap_or_log
        ( select(TAB_STEP_1).try_into()
        , &msg("HtmlElement", TAB_STEP_1)
        );
    let step_2 = unwrap_or_log
        ( select(TAB_STEP_2).try_into()
        , &msg("HtmlElement", TAB_STEP_2)
        );
    let students = unwrap_or_log
        ( select(STUDENTS).try_into()
        , &msg("TextAreaElement", STUDENTS)
        );
    let done_btn = unwrap_or_log
        ( select(DONE_BTN).try_into()
        , &msg("InputElement", DONE_BTN)
        );
    let input: InputElement = unwrap_or_log
        ( select(INPUT).try_into()
        , &msg("InputElement", INPUT)
        );
    let output: TextAreaElement = unwrap_or_log
        ( select(OUTPUT).try_into()
        , &msg("TextAreaElement", OUTPUT)
        );
    let marks_records;
    match load_marks_records() {
        Ok (mr) => {
            open_tab(&step_2);
            marks_records = Rc::new(RefCell::new(mr));
            output.set_value( &format!("{}", marks_records.borrow()) );
            input.focus();
        }
        Err (e) => {
            console!(log, e);
            open_tab(&step_1);
            marks_records = Rc::new(RefCell::new( MarksRecords::new() ));
        }
    };
    Rc::new(WebUI { 
        step_1, step_2, students, done_btn, input, output, marks_records
    })
}

fn load_event_handlers(web_ui: Rc<WebUI>) {
    let ui = web_ui.clone();
    let parse_student_list = move |_event: event::ClickEvent| {
        *ui.marks_records.borrow_mut() = 
            init_marks_records( &ui.students.value() );
        open_tab(&ui.step_2);
        ui.output.set_value
            ( &format!("{}", ui.marks_records.borrow()) );
        ui.input.focus();
        if let Err (e) =
            save_marks_records(&ui.marks_records.borrow())
        {
            console!(error, e);
        }
    };
    let ui = web_ui.clone();
    let instant_query = move |_event: event::InputEvent| {
        let s = ui.input.raw_value();
        if let Ok (Query (student_id)) = parse_input(&s) {
            if student_id.is_empty() {
                return
            }
            console!(log, "sorting students with query", &student_id);
            ui.marks_records.borrow_mut().sort_with(&student_id);
            ui.output.set_value
                ( &format!("{}", ui.marks_records.borrow()) );
        }
    };
    let ui = web_ui.clone();
    let record_or_execute = move |event: event::KeyPressEvent| {
        if "Enter" != event.key() {
            return;
        }
        console!(log, "enter key pressed");
        let s = ui.input.raw_value();
        match parse_input(&s) {
            Ok (Query (_)) => console!(log, "no marks entered"),
            Ok (MarksRecord (marks)) => {
                console!(log, "recording marks");
                if let Err (e) = 
                    ui.marks_records.borrow_mut().set_marks_at_top(marks)
                {
                    console!(error, "failed recording marks:", e);
                    return;
                }
                ui.output.set_value
                    ( &format!("{}", ui.marks_records.borrow()) );
                if let Err (e) = 
                    save_marks_records(&ui.marks_records.borrow()) 
                {
                    console!(error, e);
                }
                ui.input.set_raw_value("");
            }
            Ok (Export) => {
                console!(log, "exporting to clipboard");
                ui.output.set_value
                    (&ui.marks_records.borrow().export_string());
                js! {
                    @{&ui.output}.select();
                    document.execCommand("copy");
                }
                ui.output.set_value
                    ( "Marks copied to clipboard. \
                       Paste it into your spreadsheet."
                    );
                ui.input.set_raw_value("");
                ui.input.focus();
            }
            Ok (Clear) => {
                console!(log, "clearing data");
                ui.marks_records.borrow_mut().clear();
                ui.output.set_value("");
                ui.input.set_raw_value("");
                ui.students.set_value("");
                open_tab(&ui.step_1);
            }
            Err (e) => console!(error, "failed parsing input:", e)
        }
    };
    web_ui.done_btn.add_event_listener(parse_student_list);
    web_ui.input.add_event_listener(instant_query);
    web_ui.input.add_event_listener(record_or_execute);
}

// Log error message before unwrap panic,
// because cargo-web is currently lacking debug build.
fn unwrap_or_log<T, E: Display> (r: Result<T, E>, s: &str) -> T {
    match r {
        Ok (t)  => t,
        Err(e)  => {
            let s = format!("{}: {}", &s, e);
            console!(error, &s);
            panic!(s);
        }
    }
}

fn parse_input(s: &str) -> Result<Input, String> {
    match s {
        ":export" => Ok (Export),
        ":clear"  => Ok (Clear),
        _ if s.starts_with(':') => Err (format!("undefined escape: {}", s)),
        _ if s.contains('=') => {
            let lr: Vec<&str> = s.split('=').collect();
            if lr.len() != 2 {
                Err (format!("invalid marks input: {}", s))
            } else {
                let marks = lr[1].trim()
                    .split_whitespace()
                    .map(str::parse)
                    .collect::<Result<Marks,_>>()
                    .map_err(|e| format!("{}", e))?;
                Ok (MarksRecord (marks))
            }
        }
        _ => Ok (Query (String::from(s)))
    }
}

fn init_marks_records(s: &str) -> MarksRecords {
    let mut marks_records = MarksRecords::new();
    for line in s.lines() {
        marks_records.add_student(line.replace('\t', " "));
    }
    marks_records
}

const MARKS_RECORDS: &str = "marks_records";

fn load_marks_records() -> Result<MarksRecords, String> {
    console!(log, format!("loading {} from local storage", MARKS_RECORDS));
    match window().local_storage().get(MARKS_RECORDS) {
        Some (s) => MarksRecords::from_json_str(&s),
        None     => Err 
            (format!("local storage of {} not found", MARKS_RECORDS))
    }
}

fn save_marks_records(marks_records: &MarksRecords) -> Result<(), String> {
    console!(log, format!("saving {} to local storage", MARKS_RECORDS));
    let s = marks_records.to_json_string()?;
    if window().local_storage().insert(MARKS_RECORDS, &s).is_ok() {
        Ok (())
    } else {
        Err (format!("failed writing {} to local storage", MARKS_RECORDS))
    }
}

fn select(query: &str) -> Element {
    match document().query_selector(query) { 
        Ok(Some(element)) => element,
        _                 => {
            let msg = format!("failed querying element {}", query);
            console!(error, &msg);
            panic!(msg);
        }
    }
}

const CLASS_OPEN_TAB: &str = "open_tab";
const CLASS_CLOSED_TAB: &str = "closed_tab";

fn open_tab(tab: &HtmlElement) {
    select( &format!(".{}", CLASS_OPEN_TAB) )
        .set_attribute("class", CLASS_CLOSED_TAB).is_ok();
    tab.set_attribute("class", CLASS_OPEN_TAB).is_ok();
}
