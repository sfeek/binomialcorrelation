#![windows_subsystem = "windows"]
use fltk::{app::*, button::*, dialog::*, frame::*, input::*, output::*, window::*};

#[derive(Debug)]
// Define a struct for the form fields
struct Parameters {
    data: MultilineInput,
    robust: CheckButton,
    start_rec: IntInput,
    end_rec: IntInput,
    sd_threshold: FloatInput,
    low: Output,
    high: Output,
    current: Output,
    hyp: Frame,
    cnt: Frame,
}

fn main() {
    let app = App::default();

    // Main Window
    let mut wind = Window::new(100, 100, 500, 530, "Binomial Correlation Calculator v1.2");

    // Fill the form structure
    let mut parameters = Parameters {
        data: MultilineInput::new(16, 30, 204, 404, ""),
        robust: CheckButton::new(350, 30, 105, 21, "Robust Z"),
        start_rec: IntInput::new(350, 80, 54, 22, "Start Rec#"),
        end_rec: IntInput::new(350, 109, 54, 22, "End Rec#"),
        sd_threshold: FloatInput::new(350, 138, 54, 22, "SD Threshold"),
        low: Output::new(350, 188, 54, 22, "Low"),
        current: Output::new(350, 217, 54, 22, "Current"),
        high: Output::new(350, 246, 54, 22, "High"),
        hyp: Frame::new(325, 275, 100, 22, ""),
        cnt: Frame::new(285, 304, 200, 22, ""),
    };

    // Preset threshold
    parameters.sd_threshold.set_value("2.0");

    Frame::new(16, 10, 51, 17, "Data");

    // Calculate button
    let mut calculate_button = Button::new(18, 450, 200, 57, "Calculate");
    calculate_button.set_callback(move || calculate(&mut parameters));

    // Show the window
    wind.end();
    wind.show();

    // Enter main loop
    app.run().unwrap();
}

// Handle Calculate button
fn calculate(p: &mut Parameters) {
    // Get our start record #
    let sr: usize = match p.start_rec.value().parse::<usize>() {
        Ok(v) => v,
        Err(_) => {
            alert(368, 265, "Start Rec # Error");
            return;
        }
    };

    // Get our end record #
    let er: usize = match p.end_rec.value().parse::<usize>() {
        Ok(v) => v,
        Err(_) => {
            alert(368, 265, "End Rec # Error");
            return;
        }
    };

    // Make sure we have at least 3 values
    if er < sr + 3 {
        alert(
            368,
            265,
            "End Rec # must be at least 3 more than Start Rec # Error",
        );
        return;
    }

    // Get our SD level
    let sdt: f64 = match p.sd_threshold.value().parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            alert(368, 265, "SD Threshold Error");
            return;
        }
    };
    if sdt < 0.0 {
        alert(368, 265, "SD Threshold Error");
        return;
    }

    // Get the CSV data out of the two data fields
    let data: Vec<f64> = csv_split(&p.data.value());

    // Make sure there are enough values
    if data.len() < er {
        alert(368, 265, "End Rec # Larger Than Dataset Error");
        return;
    }

    // Calculate number of trials
    let n: usize = er - sr;

    // Z Normalize our data based on z score or robust z score
    let zn_data: Vec<f64> = z_normalize(&data, p.robust.is_checked());

    // Find all Z spike counts for the entire population
    let mut bc_total: usize = 0;
    for i in 0..zn_data.len() {
        if zn_data[i].abs() >= sdt {
            bc_total += 1;
        }
    }

    // Find all Z spikes within start and end range
    let mut bc: usize = 0;
    for i in sr..er {
        if zn_data[i].abs() >= sdt {
            bc += 1;
        }
    }

    // Calculate our values for low, high and range counts
    let pv: f64 = bc_total as f64 / zn_data.len() as f64;
    let mean: f64 = n as f64 * pv;
    let hv: f64 = (mean + (mean * (1.0 - pv)).sqrt()).ceil();
    let lv: f64 = (mean - (mean * (1.0 - pv)).sqrt()).floor();

    // Show outputs
    p.low.set_value(&f64::to_string(&lv));
    p.current.set_value(&usize::to_string(&bc));
    p.high.set_value(&f64::to_string(&hv));

    p.cnt.set_label(&format!("Trial Count = {}", n));

    if (bc as f64) < lv || (bc as f64) > hv {
        p.hyp.set_label("H0 = False");
    } else {
        p.hyp.set_label("H0 = True");
    }
}

// Convert CSV from the main windows to arrays of floats, also clean up stray whitespace
fn csv_split(inp: &String) -> Vec<f64> {
    let mut values: Vec<f64> = Vec::new();

    let clean_inp: String = inp
        .replace("\n", ",")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let fields = clean_inp.split(",");

    for f in fields {
        match f.parse::<f64>() {
            Ok(v) => values.push(v),
            Err(_) => continue,
        };
    }

    values
}

// Calculate mean
fn mean(vec: &Vec<f64>) -> f64 {
    let sum: f64 = Iterator::sum(vec.iter());
    sum / vec.len() as f64
}

// Calculate median
fn median(vec: &Vec<f64>) -> f64 {
    let mut sorted_vec = vec.to_vec();

    sorted_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n: usize = sorted_vec.len();

    if n % 2 == 0 {
        let middle: usize = n / 2 - 1;
        return (sorted_vec[middle] + sorted_vec[middle + 1]) / 2.0;
    } else {
        let middle: usize = n / 2;
        return sorted_vec[middle];
    }
}

// Calculate SD of a population
fn sd_pop(x: &Vec<f64>) -> f64 {
    let mut sd: f64 = 0.0;
    let size: usize = x.len();
    let mean = mean(&x);

    for i in 0..size {
        sd += (x[i] - mean).powf(2.0);
    }
    (sd / size as f64).sqrt()
}

// Calculate Z Scores
fn z_normalize(x: &Vec<f64>, robust: bool ) -> Vec<f64> {
    let size: usize = x.len();
    let mut zn: Vec<f64> = Vec::new();

    if robust == true {
        let mut medians: Vec<f64> = Vec::new();
        let m_s: f64 = median(x);

        for i in 0..size {
            medians.push((x[i] - m_s).abs());
        }

        let m_2: f64 = median(&medians);

        for i in 0..size {
            zn.push(0.6745 * (x[i] - m_s) / m_2);
        }
    } else {
        let avg: f64 = mean(x);
        let sd: f64 = sd_pop(x);
        for i in 0..size {
            zn.push((x[i] - avg) / sd);
        }
    }

    zn
}
