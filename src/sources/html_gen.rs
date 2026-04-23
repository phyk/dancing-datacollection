use crate::models::skating::calculate_dance_ranks;
use crate::models::{Competition, Dance, Round, RoundData};
use std::collections::BTreeMap;
use std::fmt::Write;

pub fn competition_to_html(comp: &Competition) -> String {
    let mut html = String::new();
    writeln!(html, "<!DOCTYPE html>").unwrap();
    writeln!(html, "<html>").unwrap();
    writeln!(html, "<head>").unwrap();
    writeln!(html, "  <meta charset=\"utf-8\">").unwrap();
    writeln!(html, "  <title>{}</title>", comp.name).unwrap();
    writeln!(html, "  <style>").unwrap();
    writeln!(
        html,
        "    body {{ font-family: Arial, sans-serif; font-size: 11px; color: #333; }}"
    )
    .unwrap();
    writeln!(
        html,
        "    table {{ border-collapse: collapse; width: 100%; margin-top: 10px; }}"
    )
    .unwrap();
    writeln!(html, "    th, td {{ border: 1px solid #999; padding: 3px; text-align: center; vertical-align: middle; }}").unwrap();
    writeln!(
        html,
        "    .header {{ background-color: #eee; font-weight: bold; }}"
    )
    .unwrap();
    writeln!(html, "    .left {{ text-align: left; }}").unwrap();
    writeln!(
        html,
        "    .participant {{ font-weight: bold; display: block; }}"
    )
    .unwrap();
    writeln!(
        html,
        "    .affiliation {{ font-style: italic; font-size: 9px; display: block; color: #666; }}"
    )
    .unwrap();
    writeln!(html, "    h1 {{ font-size: 16px; margin: 5px 0; }}").unwrap();
    writeln!(html, "    p {{ margin: 2px 0; }}").unwrap();
    writeln!(
        html,
        "    .total-cell {{ background-color: #f9f9f9; font-weight: bold; }}"
    )
    .unwrap();
    writeln!(html, "  </style>").unwrap();
    writeln!(html, "</head>").unwrap();
    writeln!(html, "<body>").unwrap();

    writeln!(html, "  <div class=\"eventhead\">").unwrap();
    writeln!(html, "    <h1>{}</h1>", comp.name).unwrap();
    if let Some(date) = comp.date {
        writeln!(html, "    <p>Date: {}</p>", date.format("%d.%m.%Y")).unwrap();
    }
    if let Some(ref org) = comp.organizer {
        writeln!(html, "    <p>Organizer: {}</p>", org).unwrap();
    }
    if let Some(ref club) = comp.hosting_club {
        writeln!(html, "    <p>Hosting Club: {}</p>", club).unwrap();
    }
    writeln!(html, "  </div>").unwrap();

    writeln!(html, "  <table>").unwrap();

    // Headers
    let num_judges = comp.officials.judges.len();
    writeln!(html, "    <tr class=\"header\">").unwrap();
    writeln!(html, "      <th rowspan=\"2\">Rank</th>").unwrap();
    writeln!(html, "      <th rowspan=\"2\">Participant / Club</th>").unwrap();
    writeln!(html, "      <th rowspan=\"2\">Nr</th>").unwrap();
    writeln!(html, "      <th rowspan=\"2\">R</th>").unwrap();

    for &dance in &comp.dances {
        writeln!(
            html,
            "      <th colspan=\"{}\">{}</th>",
            num_judges + 1,
            dance_name(dance)
        )
        .unwrap();
    }
    writeln!(html, "      <th rowspan=\"2\">Total</th>").unwrap();
    writeln!(html, "    </tr>").unwrap();

    writeln!(html, "    <tr class=\"header\">").unwrap();
    for _ in &comp.dances {
        for judge in &comp.officials.judges {
            writeln!(html, "      <th>{}</th>", judge.code).unwrap();
        }
        writeln!(html, "      <th>Su</th>").unwrap();
    }
    writeln!(html, "    </tr>").unwrap();

    // Data rows
    let mut sorted_participants = comp.participants.clone();
    sorted_participants.sort_by(|a, b| match (a.final_rank, b.final_rank) {
        (Some(r1), Some(r2)) => r1.cmp(&r2),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.bib_number.cmp(&b.bib_number),
    });

    for p in sorted_participants {
        let mut p_rounds: Vec<&Round> = comp
            .rounds
            .iter()
            .filter(|r| r.data.participant_bibs().contains(&p.bib_number))
            .collect();
        // Sort descending: Final (highest order) at the top of the cell
        p_rounds.sort_by_key(|r| std::cmp::Reverse(r.order));

        if p_rounds.is_empty() {
            continue;
        }

        writeln!(html, "    <tr>").unwrap();

        // Rank
        let rank_str = p
            .final_rank
            .map(|r| format!("{}.", r))
            .unwrap_or_else(|| "".to_string());
        writeln!(html, "      <td>{}</td>", rank_str).unwrap();

        // Participant / Club
        let mut name_full = p.name_one.clone();
        if let Some(ref n2) = p.name_two {
            name_full.push_str(" / ");
            name_full.push_str(n2);
        }
        let affiliation = p.affiliation.as_deref().unwrap_or("");
        writeln!(html, "      <td class=\"left\"><span class=\"participant\">{}</span><span class=\"affiliation\">{}</span></td>", name_full, affiliation).unwrap();

        // Bib
        writeln!(html, "      <td>{}</td>", p.bib_number).unwrap();

        // R (Round identifiers)
        let mut r_col = String::new();
        for (i, r) in p_rounds.iter().enumerate() {
            if i > 0 {
                r_col.push_str("<br>");
            }
            let r_id = if crate::i18n::is_final_round(&r.name) {
                "F".to_string()
            } else {
                // Use a simple heuristic or order
                (r.order + 1).to_string()
            };
            r_col.push_str(&r_id);
        }
        writeln!(html, "      <td>{}</td>", r_col).unwrap();

        // Dance columns
        for &dance in &comp.dances {
            let mut judge_cells = vec![String::new(); num_judges];
            let mut sum_cell = String::new();

            for (ri, r) in p_rounds.iter().enumerate() {
                if ri > 0 {
                    for jc in &mut judge_cells {
                        jc.push_str("<br>");
                    }
                    sum_cell.push_str("<br>");
                }

                for (ji, judge) in comp.officials.judges.iter().enumerate() {
                    judge_cells[ji].push_str(&get_mark(&r.data, &judge.code, p.bib_number, dance));
                }

                // Dance rank or sum
                match &r.data {
                    RoundData::Marking { marking_crosses } => {
                        let mut count = 0;
                        for judge in &comp.officials.judges {
                            if let Some(jm) = marking_crosses.get(&judge.code) {
                                if let Some(pm) = jm.get(&p.bib_number) {
                                    if let Some(&true) = pm.get(&dance) {
                                        count += 1;
                                    }
                                }
                            }
                        }
                        write!(sum_cell, "{}", count).unwrap();
                    }
                    RoundData::DTV { dtv_ranks } => {
                        let mut dance_marks: BTreeMap<String, BTreeMap<u32, u32>> = BTreeMap::new();
                        for (judge_code, bib_map) in dtv_ranks {
                            let mut bm = BTreeMap::new();
                            for (&bib, dm) in bib_map {
                                if let Some(&rk) = dm.get(&dance) {
                                    bm.insert(bib, rk);
                                }
                            }
                            if !bm.is_empty() {
                                dance_marks.insert(judge_code.clone(), bm);
                            }
                        }
                        let ranks = calculate_dance_ranks(&dance_marks);
                        if let Some(&rk) = ranks.get(&p.bib_number) {
                            write!(sum_cell, "{:.1}", rk).unwrap();
                        }
                    }
                    RoundData::WDSF { wdsf_scores } => {
                        let mut judge_scores = BTreeMap::new();
                        for judge in &comp.officials.judges {
                            if let Some(jm) = wdsf_scores.get(&judge.code) {
                                if let Some(pm) = jm.get(&p.bib_number) {
                                    if let Some(score) = pm.get(&dance) {
                                        judge_scores.insert(judge.code.clone(), score.clone());
                                    }
                                }
                            }
                        }
                        if !judge_scores.is_empty() {
                            let dance_score =
                                crate::models::skating::calculate_wdsf_dance_score(&judge_scores);
                            write!(sum_cell, "{:.2}", dance_score).unwrap();
                        }
                    }
                }
            }

            for jc in judge_cells {
                writeln!(html, "      <td>{}</td>", jc).unwrap();
            }
            writeln!(html, "      <td class=\"total-cell\">{}</td>", sum_cell).unwrap();
        }

        // Total column
        let mut total_col = String::new();
        for (ri, r) in p_rounds.iter().enumerate() {
            if ri > 0 {
                total_col.push_str("<br>");
            }
            let mut round_total = 0.0;
            let mut has_data = false;

            match &r.data {
                RoundData::Marking { marking_crosses } => {
                    for judge in &comp.officials.judges {
                        if let Some(jm) = marking_crosses.get(&judge.code) {
                            if let Some(pm) = jm.get(&p.bib_number) {
                                for &m in pm.values() {
                                    if m {
                                        round_total += 1.0;
                                        has_data = true;
                                    }
                                }
                            }
                        }
                    }
                    if has_data {
                        write!(total_col, "{}", round_total as u32).unwrap();
                    }
                }
                RoundData::DTV { dtv_ranks } => {
                    for &dance in &comp.dances {
                        let mut dance_marks: BTreeMap<String, BTreeMap<u32, u32>> = BTreeMap::new();
                        for (judge_code, bib_map) in dtv_ranks {
                            let mut bm = BTreeMap::new();
                            for (&bib, dm) in bib_map {
                                if let Some(&rk) = dm.get(&dance) {
                                    bm.insert(bib, rk);
                                }
                            }
                            if !bm.is_empty() {
                                dance_marks.insert(judge_code.clone(), bm);
                            }
                        }
                        let ranks = calculate_dance_ranks(&dance_marks);
                        if let Some(&rk) = ranks.get(&p.bib_number) {
                            round_total += rk;
                            has_data = true;
                        }
                    }
                    if has_data {
                        write!(total_col, "{:.1}", round_total).unwrap();
                    }
                }
                RoundData::WDSF { wdsf_scores } => {
                    for &dance in &comp.dances {
                        let mut judge_scores = BTreeMap::new();
                        for judge in &comp.officials.judges {
                            if let Some(jm) = wdsf_scores.get(&judge.code) {
                                if let Some(pm) = jm.get(&p.bib_number) {
                                    if let Some(score) = pm.get(&dance) {
                                        judge_scores.insert(judge.code.clone(), score.clone());
                                    }
                                }
                            }
                        }
                        if !judge_scores.is_empty() {
                            round_total +=
                                crate::models::skating::calculate_wdsf_dance_score(&judge_scores);
                            has_data = true;
                        }
                    }
                    if has_data {
                        write!(total_col, "{:.2}", round_total).unwrap();
                    }
                }
            }
        }
        writeln!(html, "      <td class=\"total-cell\">{}</td>", total_col).unwrap();

        writeln!(html, "    </tr>").unwrap();
    }

    writeln!(html, "  </table>").unwrap();

    // Footer with officials
    writeln!(html, "  <div class=\"officials\">").unwrap();
    writeln!(html, "    <h3>Officials</h3>").unwrap();
    writeln!(html, "    <ul>").unwrap();
    if let Some(ref rp) = comp.officials.responsible_person {
        writeln!(
            html,
            "      <li>Responsible: {} ({})</li>",
            rp.name,
            rp.club.as_deref().unwrap_or("-")
        )
        .unwrap();
    }
    if let Some(ref asst) = comp.officials.assistant {
        writeln!(
            html,
            "      <li>Assistant: {} ({})</li>",
            asst.name,
            asst.club.as_deref().unwrap_or("-")
        )
        .unwrap();
    }
    for judge in &comp.officials.judges {
        writeln!(
            html,
            "      <li>Judge {}: {} ({})</li>",
            judge.code,
            judge.name,
            judge.club.as_deref().unwrap_or("-")
        )
        .unwrap();
    }
    writeln!(html, "    </ul>").unwrap();
    writeln!(html, "  </div>").unwrap();

    writeln!(html, "</body>").unwrap();
    writeln!(html, "</html>").unwrap();

    html
}

fn dance_name(dance: Dance) -> &'static str {
    match dance {
        Dance::SlowWaltz => "Slow Waltz",
        Dance::Tango => "Tango",
        Dance::VienneseWaltz => "Viennese Waltz",
        Dance::SlowFoxtrot => "Slow Foxtrot",
        Dance::Quickstep => "Quickstep",
        Dance::Samba => "Samba",
        Dance::ChaChaCha => "Cha Cha Cha",
        Dance::Rumba => "Rumba",
        Dance::PasoDoble => "Paso Doble",
        Dance::Jive => "Jive",
    }
}

fn get_mark(data: &RoundData, judge_code: &str, bib: u32, dance: Dance) -> String {
    match data {
        RoundData::Marking { marking_crosses } => marking_crosses
            .get(judge_code)
            .and_then(|jm| jm.get(&bib))
            .and_then(|pm| pm.get(&dance))
            .map(|&m| if m { "x".to_string() } else { "-".to_string() })
            .unwrap_or_else(|| "&nbsp;".to_string()),
        RoundData::DTV { dtv_ranks } => dtv_ranks
            .get(judge_code)
            .and_then(|jm| jm.get(&bib))
            .and_then(|pm| pm.get(&dance))
            .map(|&r| r.to_string())
            .unwrap_or_else(|| "&nbsp;".to_string()),
        RoundData::WDSF { wdsf_scores } => {
            let score = wdsf_scores
                .get(judge_code)
                .and_then(|jm| jm.get(&bib))
                .and_then(|pm| pm.get(&dance));

            if let Some(s) = score {
                let mut parts = Vec::new();
                if s.technical_quality > 0.0 {
                    parts.push(format!("{:.2}", s.technical_quality));
                }
                if s.partnering_skills > 0.0 {
                    parts.push(format!("{:.2}", s.partnering_skills));
                }
                if s.movement_to_music > 0.0 {
                    parts.push(format!("{:.2}", s.movement_to_music));
                }
                if s.choreography > 0.0 {
                    parts.push(format!("{:.2}", s.choreography));
                }
                if !parts.is_empty() {
                    return parts.join("|");
                }
            }
            "&nbsp;".to_string()
        }
    }
}
