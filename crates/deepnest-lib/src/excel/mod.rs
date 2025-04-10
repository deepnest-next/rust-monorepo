use calamine::{open_workbook, DataType, Error as CalaError, Reader, Xlsx, Range, Ods};
use csv::Reader as CsvReader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

/// Repräsentiert einen Teil mit seinen Abmessungen und Eigenschaften
#[derive(Debug, Clone, Default)]
pub struct PartTable {
    pub width: f64,
    pub length: f64,
    pub thickness: Option<f64>,
    pub qty: i32,
    pub label: String,
}

/// Repräsentiert einen Fehler, der während des Lesens der Excel-Dateien auftreten kann
#[derive(Debug)]
pub enum ExcelReadError {
    CalaError(CalaError),
    IoError(io::Error),
    CsvError(csv::Error),
    MissingHeader,
    NoValidData,
    InvalidData(String),
}

impl From<CalaError> for ExcelReadError {
    fn from(error: CalaError) -> Self {
        ExcelReadError::CalaError(error)
    }
}

impl From<io::Error> for ExcelReadError {
    fn from(error: io::Error) -> Self {
        ExcelReadError::IoError(error)
    }
}

impl From<csv::Error> for ExcelReadError {
    fn from(error: csv::Error) -> Self {
        ExcelReadError::CsvError(error)
    }
}

/// Liest alle Tabellennamen aus einer Excel-Datei
/// 
/// # Arguments
/// 
/// * `file_path` - Pfad zur Excel-Datei
/// 
/// # Returns
/// 
/// Eine Liste von Tabellennamen oder einen Fehler
pub fn read_sheet_names<P: AsRef<Path>>(file_path: P) -> Result<Vec<String>, ExcelReadError> {
    let path = file_path.as_ref();
    
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => {
            let mut workbook = open_workbook(path)?;
            Ok(workbook.sheet_names().to_vec())
        },
        Some("ods") => {
            let mut workbook = calamine::open_workbook_auto(path)?;
            Ok(workbook.sheet_names().to_vec())
        },
        Some("csv") => {
            // CSV-Dateien haben nur ein einziges "Blatt"
            Ok(vec!["Sheet1".to_string()])
        },
        _ => Err(ExcelReadError::InvalidData(format!("Nicht unterstütztes Dateiformat: {:?}", path)))
    }
}

/// Liest Tabellendaten aus einer Excel-Datei oder CSV in eine PartTable-Struktur
/// 
/// # Arguments
/// 
/// * `file_path` - Pfad zur Datei
/// * `sheet_name` - Optional: Name des zu lesenden Blattes (nur für Excel-Dateien)
/// 
/// # Returns
/// 
/// Eine Liste von PartTable-Einträgen oder einen Fehler
pub fn read_parts_from_file<P: AsRef<Path>>(
    file_path: P, 
    sheet_name: Option<&str>
) -> Result<Vec<PartTable>, ExcelReadError> {
    let path = file_path.as_ref();
    
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("xlsx") | Some("xlsm") | Some("xlsb") | Some("xls") => {
            read_excel_file(path, sheet_name)
        },
        Some("ods") => {
            read_ods_file(path, sheet_name)
        },
        Some("csv") => {
            read_csv_file(path)
        },
        _ => Err(ExcelReadError::InvalidData(format!("Nicht unterstütztes Dateiformat: {:?}", path)))
    }
}

fn read_excel_file<P: AsRef<Path>>(
    file_path: P, 
    sheet_name: Option<&str>
) -> Result<Vec<PartTable>, ExcelReadError> {
    let mut workbook = open_workbook(file_path)?;
    
    // Wenn kein Blattname angegeben wurde, verwende das erste Blatt
    let sheet_name = match sheet_name {
        Some(name) => name.to_string(),
        None => workbook.sheet_names()
            .first()
            .ok_or(ExcelReadError::NoValidData)?
            .clone()
    };
    
    // Hole die Range mit den Daten
    let range = workbook.worksheet_range(&sheet_name)
        .ok_or(ExcelReadError::NoValidData)?
        .map_err(ExcelReadError::from)?;
    
    parse_range_to_parts(range)
}

fn read_ods_file<P: AsRef<Path>>(
    file_path: P, 
    sheet_name: Option<&str>
) -> Result<Vec<PartTable>, ExcelReadError> {
    let mut workbook: Ods<_> = calamine::open_workbook(file_path)?;
    
    // Wenn kein Blattname angegeben wurde, verwende das erste Blatt
    let sheet_name = match sheet_name {
        Some(name) => name.to_string(),
        None => workbook.sheet_names()
            .first()
            .ok_or(ExcelReadError::NoValidData)?
            .clone()
    };
    
    // Hole die Range mit den Daten
    let range = workbook.worksheet_range(&sheet_name)
        .ok_or(ExcelReadError::NoValidData)?
        .map_err(ExcelReadError::from)?;
    
    parse_range_to_parts(range)
}

fn read_csv_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<PartTable>, ExcelReadError> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    
    let mut csv_reader = CsvReader::from_reader(reader);
    let headers = csv_reader.headers()?.clone();
    
    let mut column_indices = HashMap::new();
    
    // Finde die relevanten Spaltenindizes
    for (i, header) in headers.iter().enumerate() {
        let header_lower = header.to_lowercase();
        
        if header_lower.contains("width") || header_lower.contains("breite") {
            column_indices.insert("width", i);
        } else if header_lower.contains("length") || header_lower.contains("länge") {
            column_indices.insert("length", i);
        } else if header_lower.contains("thickness") || header_lower.contains("dicke") {
            column_indices.insert("thickness", i);
        } else if header_lower.contains("qty") || header_lower.contains("quantity") || header_lower.contains("anzahl") {
            column_indices.insert("qty", i);
        } else if header_lower.contains("label") || header_lower.contains("name") || header_lower.contains("bezeichnung") {
            column_indices.insert("label", i);
        }
    }
    
    // Prüfe, ob die Mindestanforderungen erfüllt sind (width, length, qty)
    if !column_indices.contains_key("width") || !column_indices.contains_key("length") {
        return Err(ExcelReadError::MissingHeader);
    }
    
    let mut parts = Vec::new();
    
    for result in csv_reader.records() {
        let record = result?;
        
        let width = record.get(column_indices["width"])
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExcelReadError::InvalidData("Ungültige Breite".to_string()))?;
            
        let length = record.get(column_indices["length"])
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExcelReadError::InvalidData("Ungültige Länge".to_string()))?;
        
        let thickness = match column_indices.get("thickness") {
            Some(&idx) => record.get(idx)
                .and_then(|s| s.parse::<f64>().ok()),
            None => None
        };
        
        let qty = match column_indices.get("qty") {
            Some(&idx) => record.get(idx)
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(1),
            None => 1 // Standard-Menge ist 1
        };
        
        let label = match column_indices.get("label") {
            Some(&idx) => record.get(idx).unwrap_or("").to_string(),
            None => String::new()
        };
        
        parts.push(PartTable {
            width,
            length,
            thickness,
            qty,
            label,
        });
    }
    
    if parts.is_empty() {
        return Err(ExcelReadError::NoValidData);
    }
    
    Ok(parts)
}

fn parse_range_to_parts(range: Range<DataType>) -> Result<Vec<PartTable>, ExcelReadError> {
    if range.height() < 2 {
        return Err(ExcelReadError::NoValidData);
    }
    
    let mut column_indices = HashMap::new();
    
    // Der erste Zeilenindex ist 0 und enthält die Header
    for i in 0..range.width() {
        if let Some(DataType::String(header)) = range.get((0, i as u32)) {
            let header_lower = header.to_lowercase();
            
            if header_lower.contains("width") || header_lower.contains("breite") {
                column_indices.insert("width", i);
            } else if header_lower.contains("length") || header_lower.contains("länge") {
                column_indices.insert("length", i);
            } else if header_lower.contains("thickness") || header_lower.contains("dicke") {
                column_indices.insert("thickness", i);
            } else if header_lower.contains("qty") || header_lower.contains("quantity") || header_lower.contains("anzahl") {
                column_indices.insert("qty", i);
            } else if header_lower.contains("label") || header_lower.contains("name") || header_lower.contains("bezeichnung") {
                column_indices.insert("label", i);
            }
        }
    }
    
    // Prüfe, ob die Mindestanforderungen erfüllt sind (width, length)
    if !column_indices.contains_key("width") || !column_indices.contains_key("length") {
        return Err(ExcelReadError::MissingHeader);
    }
    
    let mut parts = Vec::new();
    
    // Starte bei Index 1 (nach der Kopfzeile)
    for row_idx in 1..range.height() {
        let width = match range.get((row_idx as u32, column_indices["width"] as u32)) {
            Some(DataType::Float(v)) => *v,
            Some(DataType::Int(v)) => *v as f64,
            _ => continue, // Überspringe Zeilen ohne gültige Breitenangabe
        };
        
        let length = match range.get((row_idx as u32, column_indices["length"] as u32)) {
            Some(DataType::Float(v)) => *v,
            Some(DataType::Int(v)) => *v as f64,
            _ => continue, // Überspringe Zeilen ohne gültige Längenangabe
        };
        
        let thickness = match column_indices.get("thickness") {
            Some(&idx) => match range.get((row_idx as u32, idx as u32)) {
                Some(DataType::Float(v)) => Some(*v),
                Some(DataType::Int(v)) => Some(*v as f64),
                _ => None,
            },
            None => None,
        };
        
        let qty = match column_indices.get("qty") {
            Some(&idx) => match range.get((row_idx as u32, idx as u32)) {
                Some(DataType::Float(v)) => *v as i32,
                Some(DataType::Int(v)) => *v as i32,
                _ => 1, // Standard-Menge ist 1
            },
            None => 1, // Standard-Menge ist 1
        };
        
        let label = match column_indices.get("label") {
            Some(&idx) => match range.get((row_idx as u32, idx as u32)) {
                Some(DataType::String(s)) => s.clone(),
                Some(DataType::Float(v)) => v.to_string(),
                Some(DataType::Int(v)) => v.to_string(),
                Some(DataType::Bool(b)) => b.to_string(),
                _ => String::new(),
            },
            None => String::new(),
        };
        
        parts.push(PartTable {
            width,
            length,
            thickness,
            qty,
            label,
        });
    }
    
    if parts.is_empty() {
        return Err(ExcelReadError::NoValidData);
    }
    
    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests können hier hinzugefügt werden
}