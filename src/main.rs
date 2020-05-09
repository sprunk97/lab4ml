use clap::clap_app;
use math::round;
use rand::{thread_rng, Rng};
use raster::{Color, Image};
use std::time::Instant;

#[derive(Clone)]
struct Point {
    x: i32,
    y: i32,
    color: Color,
}

#[derive(Clone)]
struct Cluster {
    center: Color,
    prev_center: Color,
    points: Vec<Point>,
}

// метод структуры Cluster для вычисления "расстояния" до точки по формуле близости яркости
impl Cluster {
    fn get_distance(&self, color: Color) -> i32 {
        30 * ((color.r as i32 - self.center.r as i32).pow(2))
            + 59 * ((color.g as i32 - self.center.g as i32).pow(2))
            + 11 * ((color.b as i32 - self.center.b as i32).pow(2))
    }
}

fn main() {
    //region INPUT
    // получаем количество кластеров и входной файл из аргументов запуска
    let matches = clap_app!(lab4ml =>
        (@arg CLUSTERS: -c --clusters +required +takes_value "Sets amount of clusters")
        (@arg INPUT: -i --input +required +takes_value "Sets the input file to use")
    )
    .get_matches();
    let clusters = matches
        .value_of("CLUSTERS")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let input = matches.value_of("INPUT").unwrap();
    println!("Using file: {}\r\n{} clusters", input, clusters);
    let mut im = raster::open(input).unwrap();
    //endregion
    //region RANDOM_CLUSTERS
    // генерируем кластеры со случайным "центром" - цветов
    let mut clusters = Vec::with_capacity(clusters);
    let mut rng = thread_rng();
    for _ in 0..clusters.capacity() {
        clusters.push(Cluster {
            center: Color::rgb(rng.gen(), rng.gen(), rng.gen()),
            prev_center: Color::rgb(0, 0, 0),
            points: Vec::new(),
        });
    }
    //endregion
    let now = Instant::now();
    loop {
        // вычисляем расстояние между точками и заносим точки в кластер
        calc_dist(&mut clusters, &im);
        // пересчитываем центры
        recalc_centers(&mut clusters);
        let mut change = Vec::with_capacity(clusters.len());
        // длины векторов разностей цветов между текущим и предыдущим "центрами"
        // для останова вычислений при некоторой точности
        for cl in &clusters {
            change.push(abs_sub_colors(
                &cl.center,
                &cl.prev_center,
            ));
        }
        // останов если самое значительное смещение центра не превышает количества кластеров
        change.sort_by(|a, b| b.partial_cmp(a).unwrap());
        if change[0] < clusters.len() as f32 {
            break;
        }
        // обнуляем точки перед следующей итерацией
        for cl in &mut clusters {
            cl.points = Vec::new();
        }
    }
    let elapsed = round::floor(now.elapsed().as_secs_f64(), 2);
    println!("Elapsed time: {} seconds", elapsed);
    let input = input.replace(".jpg", "");
    let len = clusters.len();
    let im = build_image(&mut im, clusters);
    let output = &format!(
        "{}{}{:?}{}{:?}{}{}",
        input, "_in_", elapsed, "s_", len, "cl", ".jpg"
    );
    match raster::save(&im, output) {
        Ok(_) => println!("Saved into {}", { output }),
        Err(err) => println!("Error saving file: {:?}", err),
    }
}

// строим выходное изображение, проходясь по точкам каждого кластера, меняя цвет на цвет центра
fn build_image<'a>(im: &'a mut Image, clusters: Vec<Cluster>) -> &'a mut Image {
    for cl in clusters {
        for p in &cl.points {
            im.set_pixel(p.x, p.y, cl.center.clone()).unwrap();
        }
    }
    im
}

fn calc_dist(clusters: &mut Vec<Cluster>, image: &Image) {
    for x in 0..image.width {
        for y in 0..image.height {
            // вектор расстояний для всех кластеров
            let mut dists = vec![0i32; clusters.len()];
            for i in 0..clusters.len() {
                dists[i] = clusters[i].get_distance(image.get_pixel(x, y).unwrap());
            }
            // определяем индекс кластера, который ближе всего к точке
            let index_ofmin = dists
                .iter()
                .position(|x| x == dists.iter().min().unwrap())
                .unwrap();
            // заносим точку в етот кластер
            clusters[index_ofmin].points.push(Point {
                x,
                y,
                color: image.get_pixel(x, y).unwrap(),
            });
        }
    }
}

// пересчитываем центры
fn recalc_centers(clusters: &mut Vec<Cluster>) {
    for cl in clusters {
        let color = average_color(&cl.points);
        cl.prev_center = Color::rgb(cl.center.r, cl.center.g, cl.center.b);
        cl.center = Color::rgb(color.0 as u8, color.1 as u8, color.2 as u8);
    }
}

// центр определяем по среднему среди всех основных цветов
fn average_color(points: &[Point]) -> (f32, f32, f32) {
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let len = points.len() as f32;
    for pt in points {
        r += pt.color.r as f32;
        g += pt.color.g as f32;
        b += pt.color.b as f32;
    }
    (r / len, g / len, b / len)
}

// разность текущего и прошлого центра, затем sqrt(r^2 + g^2 + b^2) для получившегося вектора
fn abs_sub_colors(left: &Color, right: &Color) -> f32 {
    (((left.r as i32 - right.r as i32).pow(2)
        + (left.g as i32 - right.g as i32).pow(2)
        + (left.b as i32 - right.b as i32).pow(2)) as f32)
        .sqrt()
}
