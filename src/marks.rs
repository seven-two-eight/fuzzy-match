use std::fmt;
use super::trigram;

pub type StudentId = String;
pub type Marks     = Vec<u32>;

/// Container of student marks.
#[derive(Serialize, Deserialize)]
pub struct MarksRecords {
    records: Vec<(StudentId, Marks)>
}

impl MarksRecords {
    pub fn new() -> MarksRecords {
        MarksRecords{ records: Vec::new() }
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
    pub fn insert(&mut self, student_id: StudentId, marks: Marks) {
        self.records.push((student_id, marks));
    }
    /// Update marks of the record at the top.
    pub fn set_marks_at_top(&mut self, marks: Marks) -> Result<(), String> {
        if let Some(record) = self.records.get_mut(0) {
            record.1 = marks;
        } else {
            return Err(String::from("no student record"));
        }
        Ok (())
    }
    /// Sort records by descending student id's similarity with argument `s'.
    pub fn sort_with(&mut self, s: &str) {
        self.records.sort_by_key( |(student_id, _)|
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
        let line = |(student_id, marks): &(StudentId, Marks)| format!
            ( "{}\t{}\t{}\n"
            , student_id
            , marks.iter().sum::<u32>()
            , itemize(marks)
            );
        format!
            ( "Student Id\tTotal Marks\tItem Marks\n{}"
            , self.records.iter()
                .map(line)
                .collect::<Vec<String>>()
                .concat()
            )
    }
}

impl fmt::Display for MarksRecords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (student_id, marks) in self.records.iter() {
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
        
        marks_records.insert(String::from("student A"), vec![1, 2, 3]);
        marks_records.insert(String::from("student B"), vec![4, 5, 6]);
        assert_eq!(2, marks_records.len());
        assert!(!marks_records.is_empty());
        
        marks_records.records.clear();
        assert_eq!(0, marks_records.len());
        assert!(marks_records.is_empty());
    }
    #[test]
    fn update() {
        let mut marks_records = MarksRecords::new();
        marks_records.insert(String::from("student A"), vec![1, 1, 1]);
        marks_records.insert(String::from("student B"), vec![2, 2, 2]);
        assert!(marks_records.records[0].0 == "student A");
        assert!(marks_records.records[0].1 != marks_records.records[1].1);
        
        marks_records.set_marks_at_top(vec![2, 2, 2]).unwrap();
        assert_eq!(marks_records.records[0].1, marks_records.records[1].1);
        assert_eq!(marks_records.records[0].1, vec![2, 2, 2]);
        
        marks_records.sort_with("B");
        assert!(marks_records.records[0].0 == "student B");
        assert_eq!(marks_records.records[0].1, vec![2, 2, 2]);

        marks_records.set_marks_at_top(vec![3, 3, 3]).unwrap();
        assert_eq!(marks_records.records[0].1, vec![3, 3, 3]);
        assert_eq!(marks_records.records[1].1, vec![2, 2, 2]);
    }
    #[test]
    fn serialization() {
        let mut marks_records = MarksRecords::new();
        marks_records.insert(String::from("student A"), vec![1, 1, 1]);
        marks_records.insert(String::from("student B"), vec![2, 2, 2]);
        let serialized = serde_json::to_string(&marks_records).unwrap();
        println!("MarksRecord\n{}\nserialized into json:\n{}", marks_records, serialized);
        let deserialized: MarksRecords = serde_json::from_str(&serialized).unwrap();
        assert_eq!(marks_records.records, deserialized.records);
    }
}