use std::io::Write;

use rust_code_analysis::{
    cognitive, cyclomatic, exit, fn_args, halstead, loc, mi, nom, CodeMetrics, FuncSpace,
};

#[inline(always)]
fn dump_nargs(stats: &fn_args::Stats) -> String {
    format!(
        "<details>
<summary>Nargs</summary>
<ul>
<li>Sum: {}</li>
<li>Average: {}</li>
</ul>
</details>",
        stats.nargs(),
        stats.nargs_average()
    )
}

#[inline(always)]
fn dump_nexits(stats: &exit::Stats) -> String {
    format!(
        "<details>
<summary>Nexits</summary>
<ul>
<li>Sum: {}</li>
<li>Average: {}</li>
</ul>
</details>",
        stats.exit(),
        stats.exit_average()
    )
}

#[inline(always)]
fn dump_cyclomatic(stats: &cyclomatic::Stats) -> String {
    format!(
        "<details>
<summary>Cyclomatic</summary>
<ul>
<li>Sum: {}</li>
<li>Average: {}</li>
</ul>
</details>",
        stats.cyclomatic(),
        stats.cyclomatic_average()
    )
}

#[inline(always)]
fn dump_cognitive(stats: &cognitive::Stats) -> String {
    format!(
        "<details>
<summary>Cognitive</summary>
<ul>
<li>Sum: {}</li>
<li>Average: {}</li>
</ul>
</details>",
        stats.cognitive(),
        stats.cognitive_average()
    )
}

#[inline(always)]
fn dump_loc(stats: &loc::Stats) -> String {
    format!(
        "<details>
<summary>Loc</summary>
<ul>
<li>Sloc: {}</li>
<li>Ploc: {}</li>
<li>Lloc: {}</li>
<li>Cloc: {}</li>
<li>Blank: {}</li>
</ul>
</details>",
        stats.sloc(),
        stats.ploc(),
        stats.lloc(),
        stats.cloc(),
        stats.blank()
    )
}

#[inline(always)]
fn dump_nom(stats: &nom::Stats) -> String {
    format!(
        "<details>
<summary>Nom</summary>
<ul>
<li>Functions: {}</li>
<li>Closures: {}</li>
<li>Total: {}</li>
</ul>
</details>",
        stats.functions(),
        stats.closures(),
        stats.total()
    )
}

#[inline(always)]
fn dump_halstead(stats: &halstead::Stats) -> String {
    format!(
        "<details>
<summary>Halstead</summary>
<ul>
<li>n1: {}</li>
<li>N1: {}</li>
<li>n2: {}</li>
<li>N2: {}</li>
<li>Length: {}</li>
<li>Estimated program length: {}</li>
<li>Purity ratio: {}</li>
<li>Vocabulary: {}</li>
<li>Volume: {}</li>
<li>Difficulty: {}</li>
<li>Level: {}</li>
<li>Effort: {}</li>
<li>Time: {}</li>
<li>Bugs: {}</li>
</ul>
</details>",
        stats.u_operators(),
        stats.operators(),
        stats.u_operands(),
        stats.operands(),
        stats.length(),
        stats.estimated_program_length(),
        stats.purity_ratio(),
        stats.vocabulary(),
        stats.volume(),
        stats.difficulty(),
        stats.level(),
        stats.effort(),
        stats.time(),
        stats.bugs()
    )
}

#[inline(always)]
fn dump_mi(stats: &mi::Stats) -> String {
    format!(
        "<details>
<summary>Maintainability Index</summary>
<ul>
<li>Original: {}</li>
<li>Visual studio: {}</li>
<li>Sei: {}</li>
</ul>
</details>",
        stats.mi_original(),
        stats.mi_visual_studio(),
        stats.mi_sei()
    )
}

#[inline(always)]
fn print_metrics(metrics: &CodeMetrics) -> String {
    dump_cyclomatic(&metrics.cyclomatic)
        + &dump_cognitive(&metrics.cognitive)
        + &dump_loc(&metrics.loc)
        + &dump_nom(&metrics.nom)
        + &dump_nargs(&metrics.nargs)
        + &dump_nexits(&metrics.nexits)
        + &dump_halstead(&metrics.halstead)
        + &dump_mi(&metrics.mi)
}

#[inline(always)]
fn print_spaces_name(spaces: &[FuncSpace]) -> String {
    let mut spaces_str = "<ol>\n".to_owned();
    for space in spaces {
        // Rename anonymous space
        let name = if space.name.as_ref().map_or(true, |v| v == "<anonymous>") {
            "<anonymous_space>".to_owned()
        } else {
            space.name.as_ref().unwrap().clone()
        };
        let mut s = format!(
            "<li>
<a href=\"#{}\">{}</a>",
            name, name
        );

        if !space.spaces.is_empty() {
            s += &format!("\n{}", print_spaces_name(&space.spaces));
        }

        spaces_str += &format!(
            "{}
</li>\n",
            s
        );
    }
    spaces_str + &"</ol>"
}

#[inline(always)]
fn print_spaces(spaces: &[FuncSpace]) -> String {
    let mut spaces_str = "".to_owned();
    for space in spaces {
        // Rename anonymous space
        let name = if space.name.as_ref().map_or(true, |v| v == "<anonymous>") {
            "<anonymous_space>".to_owned()
        } else {
            space.name.as_ref().unwrap().clone()
        };
        spaces_str += &format!(
            "<h3>
<span id=\"{}\">{}</span>
<span class=\"mw-editsection\">
<span class=\"mw-editsection-bracket\">[</span>
<a href=\"#\">back</a>
<span class=\"mw-editsection-bracket\">]</span>
</span>
</h3>
{}",
            name,
            name,
            print_metrics(&space.metrics)
        );

        if !space.spaces.is_empty() {
            spaces_str += &print_spaces(&space.spaces);
        }
    }
    spaces_str
}

#[inline(always)]
fn global_metrics(metrics: &CodeMetrics) -> String {
    format!(
        "<h1><span id=\"Global_metrics\">Global metrics</span></h1>\n{}",
        print_metrics(metrics)
    )
}

#[inline(always)]
fn spaces(space: &FuncSpace) -> String {
    format!(
        "<h1><span id=\"Spaces\">Spaces</span></h1>\n{}",
        print_spaces(&space.spaces)
    )
}

#[inline(always)]
fn index(space: &FuncSpace) -> String {
    let const_index = "<div id=\"toc_container\">
<p class=\"toc_title\">Contents</p>
<ol>
<li>
<a href=\"#Global_metrics\">Global metrics</a>
</li>";

    let space_index = if !space.spaces.is_empty() {
        format!(
            "<li>
<a href=\"#Spaces\">Spaces</a>
{}
</li>",
            print_spaces_name(&space.spaces)
        )
    } else {
        "".to_owned()
    };

    format!(
        "{}
{}
</ol>
</div>",
        const_index, space_index
    )
}

pub(crate) fn to_string(space: &FuncSpace) -> std::io::Result<String> {
    let filename = space.name.as_ref().unwrap();
    let css_data =
        std::str::from_utf8(include_bytes!(concat!("../resources/", "rca.css"))).unwrap();

    let head = format!(
        "<head>
<title>{} metrics</title>
<meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\">
<style>
{}
</style>
</head>",
        filename, css_data
    );

    let space_h1 = if !space.spaces.is_empty() {
        spaces(space)
    } else {
        "".to_owned()
    };

    let body = format!(
        "<body>
{}
{}
{}
</body>",
        index(space),
        global_metrics(&space.metrics),
        space_h1
    );

    Ok(format!(
        "<!DOCTYPE html>
<html lang=\"en-us\">
{}
{}
</html>",
        head, body
    ))
}

pub(crate) fn to_writer<W: Write>(file: &mut W, space: &FuncSpace) -> std::io::Result<()> {
    let html_data = to_string(space).unwrap();
    file.write_all(html_data.as_bytes())
}
