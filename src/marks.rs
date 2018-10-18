use std::fmt;
use super::trigram;

pub type RecordId  = u32;
pub type StudentId = String;
pub type Marks     = Vec<u32>;

const NULL_RECORD_ID: RecordId  = 0;
const FIRST_RECORD_ID: RecordId = 1;

/// Container of student marks.
#[derive(Serialize, Deserialize)]
pub struct MarksRecords {
    next_record_id: RecordId,
    records: Vec<(RecordId, StudentId, Marks)>
}

impl MarksRecords {
    pub fn new() -> MarksRecords {
        MarksRecords{ next_record_id: FIRST_RECORD_ID, records: Vec::new() }
    }
    #[allow(unused)]
    pub fn len(&self) -> usize {
        self.records.len()
    }
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
    pub fn clear(&mut self) {
        self.records.clear();
    }
    /// Add student with empty marks.
    pub fn add_student (&mut self, student_id: StudentId)
    {
        self.records.push((NULL_RECORD_ID, student_id, vec![]));
    }
    /// Update marks of the record at the top.
    pub fn set_marks_at_top(&mut self, marks: Marks) -> Result<(), String> {
        if let Some(record) = self.records.get_mut(0) {
            record.2 = marks;
            if record.0 == NULL_RECORD_ID {
                record.0 = self.next_record_id;
                self.next_record_id += 1;
            }
        } else {
            return Err(String::from("no student record"));
        }
        Ok (())
    }
    /// Sort records by descending student id's similarity with argument `s`.
    pub fn sort_with(&mut self, s: &str) {
        self.records.sort_by_key( |(_, student_id, _)|
            (trigram::score(student_id, s) * -1e5) as i32
        );
    }
    pub fn to_json_string(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e|
            format!("failed serialization: {}", e)
        )
    }
    pub fn from_json_str(s: &str) -> Result<MarksRecords, String> {
        serde_json::from_str(s).map_err(|e|
            format!("failed deserializing {}: {}", s, e)
        )
    }
    pub fn export_string(&self) -> String {
        let itemize = |marks: &Marks| marks.iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join("\t");
        let line = |(record_id, student_id, marks)
                : &(RecordId, StudentId, Marks)| 
            format!
                ( "{}\t{}\t{}\t{}\n"
                , record_id
                , student_id
                , marks.iter().sum::<u32>()
                , itemize(marks)
                );
        format!
            ( "Record Id\tStudent Id\tTotal Marks\tItem Marks\n{}"
            , self.records.iter()
                .map(line)
                .collect::<Vec<String>>()
                .concat()
            )
    }
}

impl fmt::Display for MarksRecords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (record_id, student_id, marks) in self.records.iter() {
            if *record_id == NULL_RECORD_ID {
                write!(f, "    ");
            } else {
                write!(f, "{:<4.4}", record_id);
            }
            write!(f, "{:24.24}", student_id)?;
            if !marks.is_empty() {
                let sum: u32 = marks.iter().sum();
                write!(f, " {:>10} = {:?}", sum, marks)?;
            }
            write!(f, "\n")?;
        }
        Ok (())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        let mut marks_records = MarksRecords::new();
        assert_eq!(0, marks_records.len());
        assert!(marks_records.is_empty());
        
        marks_records.add_student(String::from("student A"));
        marks_records.add_student(String::from("student B"));
        assert_eq!(2, marks_records.len());
        assert!(!marks_records.is_empty());
        
        marks_records.records.clear();
        assert_eq!(0, marks_records.len());
        assert!(marks_records.is_empty());
    }
    #[test]
    fn update() {
        let mut marks_records = MarksRecords::new();
        marks_records.add_student(String::from("student A"));
        marks_records.add_student(String::from("student B"));
        marks_records.set_marks_at_top(vec![1, 1, 1]).unwrap();
        assert!(marks_records.records[0].0 == FIRST_RECORD_ID);
        assert!(marks_records.records[0].1 == "student A");
        assert!(marks_records.records[1].0 == NULL_RECORD_ID);

        marks_records.sort_with("B");
        assert!(marks_records.records[0].1 == "student B");
        marks_records.set_marks_at_top(vec![2, 2, 2]).unwrap();
        assert!(marks_records.records[0].2 != marks_records.records[1].2);
        assert!(marks_records.records[0].0 == FIRST_RECORD_ID + 1);
        
        marks_records.sort_with("A");
        assert!(marks_records.records[0].1 == "student A");
        marks_records.set_marks_at_top(vec![2, 2, 2]).unwrap();
        assert_eq!(marks_records.records[0].2, marks_records.records[1].2);
        assert_eq!(marks_records.records[0].2, vec![2, 2, 2]);
        assert!(marks_records.records[0].0 == FIRST_RECORD_ID);

        marks_records.set_marks_at_top(vec![3, 3, 3]).unwrap();
        assert_eq!(marks_records.records[0].2, vec![3, 3, 3]);
        assert_eq!(marks_records.records[1].2, vec![2, 2, 2]);
        assert!(marks_records.records[1].0 == FIRST_RECORD_ID + 1);
    }
    #[test]
    fn serialization() {
        let mut marks_records = MarksRecords::new();
        marks_records.add_student(String::from("student A"));
        marks_records.set_marks_at_top(vec![1, 1, 1]).unwrap();
        marks_records.add_student(String::from("student B"));
        marks_records.sort_with("B");
        marks_records.set_marks_at_top(vec![2, 2, 2]).unwrap();
        let serialized = serde_json::to_string(&marks_records).unwrap();
        println!("MarksRecord\n{}\nserialized into json:\n{}", marks_records, serialized);
        let deserialized: MarksRecords = serde_json::from_str(&serialized).unwrap();
        assert_eq!(marks_records.records, deserialized.records);
    }
}