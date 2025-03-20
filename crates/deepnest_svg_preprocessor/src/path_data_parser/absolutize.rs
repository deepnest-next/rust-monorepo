use super::Segment;

pub fn absolutize(segments: Vec<Segment>) -> Vec<Segment> {
    let mut cx = 0.0;
    let mut cy = 0.0;
    let mut subx = 0.0;
    let mut suby = 0.0;
    let mut out: Vec<Segment> = Vec::new();

    for seg in segments {
        match seg.key {
            'M' => {
                out.push(Segment {
                    key: 'M',
                    data: seg.data.clone(),
                });
                if seg.data.len() >= 2 {
                    cx = seg.data[0];
                    cy = seg.data[1];
                    subx = cx;
                    suby = cy;
                }
            }
            'm' => {
                if seg.data.len() >= 2 {
                    cx += seg.data[0];
                    cy += seg.data[1];
                    out.push(Segment {
                        key: 'M',
                        data: vec![cx, cy],
                    });
                    subx = cx;
                    suby = cy;
                }
            }
            'L' => {
                out.push(Segment {
                    key: 'L',
                    data: seg.data.clone(),
                });
                if seg.data.len() >= 2 {
                    cx = seg.data[0];
                    cy = seg.data[1];
                }
            }
            'l' => {
                if seg.data.len() >= 2 {
                    cx += seg.data[0];
                    cy += seg.data[1];
                    out.push(Segment {
                        key: 'L',
                        data: vec![cx, cy],
                    });
                }
            }
            'C' => {
                out.push(Segment {
                    key: 'C',
                    data: seg.data.clone(),
                });
                if seg.data.len() >= 6 {
                    cx = seg.data[4];
                    cy = seg.data[5];
                }
            }
            'c' => {
                let newdata: Vec<f64> = seg
                    .data
                    .iter()
                    .enumerate()
                    .map(|(i, &d)| if i % 2 == 0 { d + cx } else { d + cy })
                    .collect();
                out.push(Segment {
                    key: 'C',
                    data: newdata.clone(),
                });
                if newdata.len() >= 6 {
                    cx = newdata[4];
                    cy = newdata[5];
                }
            }
            'Q' => {
                out.push(Segment {
                    key: 'Q',
                    data: seg.data.clone(),
                });
                if seg.data.len() >= 4 {
                    cx = seg.data[2];
                    cy = seg.data[3];
                }
            }
            'q' => {
                let newdata: Vec<f64> = seg
                    .data
                    .iter()
                    .enumerate()
                    .map(|(i, &d)| if i % 2 == 0 { d + cx } else { d + cy })
                    .collect();
                out.push(Segment {
                    key: 'Q',
                    data: newdata.clone(),
                });
                if newdata.len() >= 4 {
                    cx = newdata[2];
                    cy = newdata[3];
                }
            }
            'A' => {
                out.push(Segment {
                    key: 'A',
                    data: seg.data.clone(),
                });
                if seg.data.len() >= 7 {
                    cx = seg.data[5];
                    cy = seg.data[6];
                }
            }
            'a' => {
                if seg.data.len() >= 7 {
                    cx += seg.data[5];
                    cy += seg.data[6];
                    let newdata = vec![
                        seg.data[0],
                        seg.data[1],
                        seg.data[2],
                        seg.data[3],
                        seg.data[4],
                        cx,
                        cy,
                    ];
                    out.push(Segment {
                        key: 'A',
                        data: newdata,
                    });
                }
            }
            'H' => {
                out.push(Segment {
                    key: 'H',
                    data: seg.data.clone(),
                });
                if !seg.data.is_empty() {
                    cx = seg.data[0];
                }
            }
            'h' => {
                if !seg.data.is_empty() {
                    cx += seg.data[0];
                    out.push(Segment {
                        key: 'H',
                        data: vec![cx],
                    });
                }
            }
            'V' => {
                out.push(Segment {
                    key: 'V',
                    data: seg.data.clone(),
                });
                if !seg.data.is_empty() {
                    cy = seg.data[0];
                }
            }
            'v' => {
                if !seg.data.is_empty() {
                    cy += seg.data[0];
                    out.push(Segment {
                        key: 'V',
                        data: vec![cy],
                    });
                }
            }
            'S' => {
                out.push(Segment {
                    key: 'S',
                    data: seg.data.clone(),
                });
                if seg.data.len() >= 4 {
                    cx = seg.data[2];
                    cy = seg.data[3];
                }
            }
            's' => {
                let newdata: Vec<f64> = seg
                    .data
                    .iter()
                    .enumerate()
                    .map(|(i, &d)| if i % 2 == 0 { d + cx } else { d + cy })
                    .collect();
                out.push(Segment {
                    key: 'S',
                    data: newdata.clone(),
                });
                if newdata.len() >= 4 {
                    cx = newdata[2];
                    cy = newdata[3];
                }
            }
            'T' => {
                out.push(Segment {
                    key: 'T',
                    data: seg.data.clone(),
                });
                if seg.data.len() >= 2 {
                    cx = seg.data[0];
                    cy = seg.data[1];
                }
            }
            't' => {
                if seg.data.len() >= 2 {
                    cx += seg.data[0];
                    cy += seg.data[1];
                    out.push(Segment {
                        key: 'T',
                        data: vec![cx, cy],
                    });
                }
            }
            'Z' | 'z' => {
                out.push(Segment {
                    key: 'Z',
                    data: Vec::new(),
                });
                cx = subx;
                cy = suby;
            }
            _ => {
                // Unbekannter Befehl: entweder ignorieren oder unverändert übernehmen.
            }
        }
    }
    out
}