export type TransformTemplateSource = "builtin" | "custom";

export interface TransformTemplate {
  id: string;
  source: TransformTemplateSource;
  name: string;
  category: string;
  transform: string;
  summary: string;
  body: string;
  tags: string[];
}

export type CustomTransformTemplate = Omit<TransformTemplate, "source">;

export interface TransformTemplateFillField {
  name: string;
  value: string;
  source: "calc-assignment" | "structured-field";
}

export interface TransformTemplateAssistance {
  stepId: string;
  stepLabel: string;
  suggestedAnswer: string;
  rationale: string;
  contextSignals: string[];
  actionLabel: string;
}

export interface TransformTemplateAssistanceInput {
  templates: TransformTemplate[];
  filteredTemplates: TransformTemplate[];
  query?: string | null;
  category?: string | null;
  transform?: string | null;
  documentText?: string | null;
  customTemplate?: Partial<CustomTransformTemplate> | null;
  assistanceNotes?: string | null;
}

const fenceTicks = "```";

function fenced(transform: string, content: string, options = "") {
  const suffix = options.trim() ? ` ${options.trim()}` : "";
  return `${fenceTicks}${transform}${suffix}\n${content.trim()}\n${fenceTicks}\n`;
}

function calc(content: string, after = "") {
  return `${fenced("calc", content)}${after}`;
}

function template(
  id: string,
  category: string,
  transform: string,
  name: string,
  summary: string,
  body: string,
  tags: string[] = [],
): TransformTemplate {
  return { id, source: "builtin", category, transform, name, summary, body, tags };
}

export const builtinTransformTemplates: TransformTemplate[] = [
  template(
    "calc-business-roi",
    "Business",
    "calc",
    "ROI and payback",
    "Measures net return, ROI, and simple payback period for an investment.",
    calc(
      `
investment = 125000
annual_benefit = 52000
annual_cost = 9000
net_annual_benefit = annual_benefit - annual_cost
roi = net_annual_benefit / investment
payback_years = investment / net_annual_benefit
`,
      "ROI: {{=roi | percent}}\nPayback: {{=payback_years}} years\n",
    ),
    ["roi", "investment", "payback"],
  ),
  template(
    "calc-business-break-even",
    "Business",
    "calc",
    "Break-even volume",
    "Finds contribution margin and units required to cover fixed cost.",
    calc(
      `
fixed_cost = 85000
price_per_unit = 120
variable_cost_per_unit = 62
contribution_margin = price_per_unit - variable_cost_per_unit
break_even_units = fixed_cost / contribution_margin
`,
      "Break-even units: {{=break_even_units}}\n",
    ),
    ["pricing", "margin", "break-even"],
  ),
  template(
    "calc-business-gross-margin",
    "Business",
    "calc",
    "Gross margin bridge",
    "Calculates revenue, gross profit, gross margin, and cost share.",
    calc(
      `
units = 4200
price = 49
cost_per_unit = 21
revenue = units * price
cogs = units * cost_per_unit
gross_profit = revenue - cogs
gross_margin = gross_profit / revenue
cogs_share = cogs / revenue
`,
      "Gross margin: {{=gross_margin | percent}}\n",
    ),
    ["revenue", "gross-margin"],
  ),
  template(
    "calc-business-saas-unit-economics",
    "Business",
    "calc",
    "SaaS unit economics",
    "Calculates ARR, gross margin contribution, LTV, CAC payback, and LTV/CAC.",
    calc(
      `
monthly_arpu = 84
customers = 1600
gross_margin = 0.78
monthly_churn = 0.025
cac = 520
mrr = monthly_arpu * customers
arr = mrr * 12
ltv = monthly_arpu * gross_margin / monthly_churn
ltv_to_cac = ltv / cac
cac_payback_months = cac / (monthly_arpu * gross_margin)
`,
      "ARR: {{=arr | currency}}\nLTV/CAC: {{=ltv_to_cac}}\nCAC payback: {{=cac_payback_months}} months\n",
    ),
    ["saas", "ltv", "cac"],
  ),
  template(
    "calc-business-runway",
    "Business",
    "calc",
    "Cash runway",
    "Estimates monthly burn, runway, and funding gap.",
    calc(
      `
cash_on_hand = 620000
monthly_revenue = 95000
monthly_payroll = 210000
monthly_opex = 76000
monthly_burn = monthly_payroll + monthly_opex - monthly_revenue
runway_months = cash_on_hand / monthly_burn
target_runway_months = 18
funding_gap = target_runway_months * monthly_burn - cash_on_hand
`,
      "Runway: {{=runway_months}} months\nFunding gap: {{=funding_gap | currency}}\n",
    ),
    ["cash", "runway", "burn"],
  ),
  template(
    "calc-business-npv",
    "Business",
    "calc",
    "Three-year NPV",
    "Discounts three annual cash flows without requiring exponent syntax.",
    calc(
      `
initial_investment = 250000
discount_rate = 0.11
year_1_cash = 90000
year_2_cash = 120000
year_3_cash = 155000
discount_1 = 1 + discount_rate
discount_2 = discount_1 * discount_1
discount_3 = discount_2 * discount_1
present_value = year_1_cash / discount_1 + year_2_cash / discount_2 + year_3_cash / discount_3
npv = present_value - initial_investment
`,
      "NPV: {{=npv | currency}}\n",
    ),
    ["npv", "discounted-cash-flow"],
  ),
  template(
    "calc-business-cagr",
    "Business",
    "calc",
    "Two-period CAGR",
    "Estimates two-year compounded growth using a square-root approximation input.",
    calc(
      `
start_value = 1200000
end_value = 1800000
sqrt_growth_multiple = 1.2247
cagr = sqrt_growth_multiple - 1
absolute_growth = end_value - start_value
`,
      "CAGR: {{=cagr | percent}}\nGrowth: {{=absolute_growth | currency}}\n",
    ),
    ["growth", "cagr"],
  ),
  template(
    "calc-business-pricing-sensitivity",
    "Business",
    "calc",
    "Pricing sensitivity",
    "Compares base, discount, and premium price cases.",
    calc(
      `
base_units = 10000
base_price = 35
unit_cost = 18
discount_price = base_price * 0.9
premium_price = base_price * 1.08
discount_units = base_units * 1.14
premium_units = base_units * 0.94
base_profit = (base_price - unit_cost) * base_units
discount_profit = (discount_price - unit_cost) * discount_units
premium_profit = (premium_price - unit_cost) * premium_units
best_case_profit = MAX(base_profit, discount_profit, premium_profit)
`,
      "Best case profit: {{=best_case_profit | currency}}\n",
    ),
    ["pricing", "sensitivity"],
  ),
  template(
    "calc-business-sales-pipeline",
    "Business",
    "calc",
    "Sales pipeline forecast",
    "Weights pipeline by stage probability and compares it to target.",
    calc(
      `
target = 1200000
qualified = 950000
proposal = 620000
commit = 280000
qualified_probability = 0.25
proposal_probability = 0.55
commit_probability = 0.85
weighted_pipeline = qualified * qualified_probability + proposal * proposal_probability + commit * commit_probability
coverage = weighted_pipeline / target
gap = target - weighted_pipeline
`,
      "Pipeline coverage: {{=coverage | percent}}\nGap: {{=gap | currency}}\n",
    ),
    ["sales", "pipeline"],
  ),
  template(
    "calc-business-churn-retention",
    "Business",
    "calc",
    "Churn and retention",
    "Calculates customer churn, net retention, and expansion contribution.",
    calc(
      `
starting_arr = 2400000
lost_arr = 180000
contraction_arr = 70000
expansion_arr = 260000
ending_arr = starting_arr - lost_arr - contraction_arr + expansion_arr
gross_retention = (starting_arr - lost_arr - contraction_arr) / starting_arr
net_retention = ending_arr / starting_arr
churn_rate = lost_arr / starting_arr
`,
      "Gross retention: {{=gross_retention | percent}}\nNet retention: {{=net_retention | percent}}\n",
    ),
    ["retention", "churn"],
  ),
  template(
    "calc-business-inventory-reorder",
    "Business",
    "calc",
    "Inventory reorder point",
    "Calculates reorder point with demand, lead time, and safety stock.",
    calc(
      `
daily_demand = 340
lead_time_days = 12
safety_stock = 900
current_inventory = 5200
reorder_point = daily_demand * lead_time_days + safety_stock
days_until_reorder = (current_inventory - reorder_point) / daily_demand
should_reorder = IF(current_inventory <= reorder_point, 1, 0)
`,
      "Reorder point: {{=reorder_point}}\nDays until reorder: {{=days_until_reorder}}\n",
    ),
    ["inventory", "operations"],
  ),
  template(
    "calc-business-capacity",
    "Business",
    "calc",
    "Capacity utilization",
    "Compares demand against available operating capacity.",
    calc(
      `
stations = 8
hours_per_day = 7.5
units_per_hour = 42
operating_days = 22
monthly_capacity = stations * hours_per_day * units_per_hour * operating_days
forecast_demand = 51500
utilization = forecast_demand / monthly_capacity
capacity_gap = forecast_demand - monthly_capacity
`,
      "Utilization: {{=utilization | percent}}\nCapacity gap: {{=capacity_gap}}\n",
    ),
    ["capacity", "operations"],
  ),
  template(
    "calc-business-variance",
    "Business",
    "calc",
    "Budget variance",
    "Calculates absolute and percentage variance against budget.",
    calc(
      `
budget = 480000
actual = 512000
variance = actual - budget
variance_percent = variance / budget
status = IF(actual <= budget, 1, 0)
`,
      "Variance: {{=variance | currency}}\nVariance percent: {{=variance_percent | percent}}\n",
    ),
    ["budget", "variance"],
  ),
  template(
    "calc-business-weighted-score",
    "Business",
    "calc",
    "Weighted decision score",
    "Scores an option across value, feasibility, risk, and speed.",
    calc(
      `
value_score = 8
feasibility_score = 6
risk_score = 4
speed_score = 7
value_weight = 0.4
feasibility_weight = 0.25
risk_weight = 0.2
speed_weight = 0.15
weighted_score = value_score * value_weight + feasibility_score * feasibility_weight + (10 - risk_score) * risk_weight + speed_score * speed_weight
`,
      "Weighted score: {{=weighted_score}}\n",
    ),
    ["decision", "score"],
  ),
  template(
    "calc-business-kpi-index",
    "Business",
    "calc",
    "Composite KPI index",
    "Normalizes several KPIs against targets into one index.",
    calc(
      `
revenue_actual = 1280000
revenue_target = 1200000
margin_actual = 0.58
margin_target = 0.55
nps_actual = 48
nps_target = 45
delivery_actual = 0.92
delivery_target = 0.95
kpi_index = AVG(revenue_actual / revenue_target, margin_actual / margin_target, nps_actual / nps_target, delivery_actual / delivery_target)
`,
      "KPI index: {{=kpi_index | percent}}\n",
    ),
    ["kpi", "scorecard"],
  ),
  template(
    "calc-science-dilution",
    "Science",
    "calc",
    "Dilution planner",
    "Uses C1V1 = C2V2 to calculate stock volume and solvent volume.",
    calc(
      `
stock_concentration = 10
target_concentration = 2
final_volume_ml = 50
stock_volume_ml = target_concentration * final_volume_ml / stock_concentration
solvent_volume_ml = final_volume_ml - stock_volume_ml
`,
      "Stock volume: {{=stock_volume_ml}} mL\nSolvent volume: {{=solvent_volume_ml}} mL\n",
    ),
    ["chemistry", "dilution"],
  ),
  template(
    "calc-science-molarity",
    "Science",
    "calc",
    "Molarity",
    "Calculates moles and molarity from mass, molar mass, and volume.",
    calc(
      `
mass_g = 5.85
molar_mass_g_per_mol = 58.44
volume_l = 0.5
moles = mass_g / molar_mass_g_per_mol
molarity = moles / volume_l
`,
      "Molarity: {{=molarity}} mol/L\n",
    ),
    ["chemistry", "molarity"],
  ),
  template(
    "calc-science-ideal-gas",
    "Science",
    "calc",
    "Ideal gas",
    "Calculates pressure from moles, gas constant, temperature, and volume.",
    calc(
      `
moles = 1.2
gas_constant = 0.082057
temperature_k = 298
volume_l = 24
pressure_atm = moles * gas_constant * temperature_k / volume_l
`,
      "Pressure: {{=pressure_atm}} atm\n",
    ),
    ["physics", "chemistry"],
  ),
  template(
    "calc-science-half-life",
    "Science",
    "calc",
    "Two half-life decay",
    "Calculates remaining quantity after two half-life periods.",
    calc(
      `
initial_quantity = 100
elapsed_half_lives = 2
remaining_fraction = 0.25
remaining_quantity = initial_quantity * remaining_fraction
decayed_quantity = initial_quantity - remaining_quantity
`,
      "Remaining: {{=remaining_quantity}}\nDecayed: {{=decayed_quantity}}\n",
    ),
    ["decay", "physics"],
  ),
  template(
    "calc-science-ohms-law",
    "Science",
    "calc",
    "Ohm power",
    "Calculates voltage, current, resistance, and power relationships.",
    calc(
      `
current_a = 2.4
resistance_ohm = 18
voltage_v = current_a * resistance_ohm
power_w = voltage_v * current_a
`,
      "Voltage: {{=voltage_v}} V\nPower: {{=power_w}} W\n",
    ),
    ["electronics", "physics"],
  ),
  template(
    "calc-science-density",
    "Science",
    "calc",
    "Density",
    "Calculates density and relative percentage difference from reference.",
    calc(
      `
mass_g = 271
volume_cm3 = 100
density = mass_g / volume_cm3
reference_density = 2.70
difference = density - reference_density
percent_difference = difference / reference_density
`,
      "Density: {{=density}} g/cm3\nDifference: {{=percent_difference | percent}}\n",
    ),
    ["physics", "materials"],
  ),
  template(
    "calc-science-kinetic-energy",
    "Science",
    "calc",
    "Kinetic energy",
    "Calculates kinetic energy from mass and velocity.",
    calc(
      `
mass_kg = 12
velocity_m_s = 7.5
kinetic_energy_j = 0.5 * mass_kg * velocity_m_s * velocity_m_s
`,
      "Kinetic energy: {{=kinetic_energy_j}} J\n",
    ),
    ["physics", "energy"],
  ),
  template(
    "calc-science-concentration-change",
    "Science",
    "calc",
    "Concentration change",
    "Compares baseline and observed concentration values.",
    calc(
      `
baseline = 18.5
observed = 22.4
absolute_change = observed - baseline
relative_change = absolute_change / baseline
is_increase = IF(observed > baseline, 1, 0)
`,
      "Relative change: {{=relative_change | percent}}\n",
    ),
    ["experiment", "change"],
  ),
  template(
    "calc-science-replicate-summary",
    "Science",
    "calc",
    "Replicate summary",
    "Calculates average, range, and percent range for three readings.",
    calc(
      `
reading_1 = 4.8
reading_2 = 5.1
reading_3 = 4.9
mean = AVG(reading_1, reading_2, reading_3)
range = MAX(reading_1, reading_2, reading_3) - MIN(reading_1, reading_2, reading_3)
range_percent = range / mean
`,
      "Mean: {{=mean}}\nRange percent: {{=range_percent | percent}}\n",
    ),
    ["experiment", "replicates"],
  ),
  template(
    "calc-science-dose",
    "Science",
    "calc",
    "Dose by weight",
    "Calculates dose amount from body weight and dose rate.",
    calc(
      `
weight_kg = 72
dose_mg_per_kg = 5
total_dose_mg = weight_kg * dose_mg_per_kg
tablet_strength_mg = 100
tablet_count = total_dose_mg / tablet_strength_mg
`,
      "Total dose: {{=total_dose_mg}} mg\nTablets: {{=tablet_count}}\n",
    ),
    ["clinical", "dose"],
  ),
  template(
    "calc-math-linear",
    "Mathematics",
    "calc",
    "Linear model",
    "Evaluates y = mx + b and residual error.",
    calc(
      `
slope = 2.4
intercept = 8
x = 15
observed_y = 45
predicted_y = slope * x + intercept
residual = observed_y - predicted_y
`,
      "Predicted y: {{=predicted_y}}\nResidual: {{=residual}}\n",
    ),
    ["algebra", "model"],
  ),
  template(
    "calc-math-quadratic",
    "Mathematics",
    "calc",
    "Quadratic value",
    "Evaluates ax^2 + bx + c at a selected x.",
    calc(
      `
a = 1
b = -3
c = 2
x = 4
y = a * x * x + b * x + c
`,
      "y: {{=y}}\n",
    ),
    ["algebra", "quadratic"],
  ),
  template(
    "calc-math-distance",
    "Mathematics",
    "calc",
    "Squared distance",
    "Calculates dx, dy, and squared distance for two points.",
    calc(
      `
x1 = 2
y1 = 5
x2 = 11
y2 = 17
dx = x2 - x1
dy = y2 - y1
distance_squared = dx * dx + dy * dy
`,
      "Distance squared: {{=distance_squared}}\n",
    ),
    ["geometry", "distance"],
  ),
  template(
    "calc-math-weighted-average",
    "Mathematics",
    "calc",
    "Weighted average",
    "Calculates a weighted average across three values.",
    calc(
      `
value_a = 78
value_b = 91
value_c = 84
weight_a = 0.25
weight_b = 0.5
weight_c = 0.25
weighted_average = value_a * weight_a + value_b * weight_b + value_c * weight_c
`,
      "Weighted average: {{=weighted_average}}\n",
    ),
    ["statistics", "average"],
  ),
  template(
    "calc-math-bayes",
    "Mathematics",
    "calc",
    "Bayes update",
    "Calculates posterior probability from prior, sensitivity, and false-positive rate.",
    calc(
      `
prior = 0.08
sensitivity = 0.92
false_positive_rate = 0.06
true_positive = sensitivity * prior
false_positive = false_positive_rate * (1 - prior)
posterior = true_positive / (true_positive + false_positive)
`,
      "Posterior probability: {{=posterior | percent}}\n",
    ),
    ["probability", "bayes"],
  ),
  template(
    "calc-math-expected-value",
    "Mathematics",
    "calc",
    "Expected value",
    "Calculates expected value for three scenarios.",
    calc(
      `
downside_value = -40000
base_value = 85000
upside_value = 180000
downside_probability = 0.2
base_probability = 0.55
upside_probability = 0.25
expected_value = downside_value * downside_probability + base_value * base_probability + upside_value * upside_probability
`,
      "Expected value: {{=expected_value | currency}}\n",
    ),
    ["probability", "expected-value"],
  ),
  template(
    "calc-math-normalized-score",
    "Mathematics",
    "calc",
    "Normalized score",
    "Normalizes a value between minimum and maximum bounds.",
    calc(
      `
value = 72
minimum = 40
maximum = 95
normalized = (value - minimum) / (maximum - minimum)
score_100 = normalized * 100
`,
      "Normalized score: {{=score_100}}\n",
    ),
    ["normalization", "score"],
  ),
  template(
    "calc-math-compound-growth",
    "Mathematics",
    "calc",
    "Three-period compound growth",
    "Compounds a value for three equal periods.",
    calc(
      `
initial_value = 10000
growth_rate = 0.08
period_1 = initial_value * (1 + growth_rate)
period_2 = period_1 * (1 + growth_rate)
period_3 = period_2 * (1 + growth_rate)
total_growth = period_3 - initial_value
`,
      "Period 3: {{=period_3 | currency}}\nTotal growth: {{=total_growth | currency}}\n",
    ),
    ["growth", "compound"],
  ),
  template(
    "calc-math-error",
    "Mathematics",
    "calc",
    "Measurement error",
    "Calculates absolute error and relative error.",
    calc(
      `
measured = 99.2
accepted = 100
absolute_error = ABS(measured - accepted)
relative_error = absolute_error / accepted
`,
      "Relative error: {{=relative_error | percent}}\n",
    ),
    ["error", "measurement"],
  ),
  template(
    "calc-math-index",
    "Mathematics",
    "calc",
    "Index change",
    "Calculates index level and change against a base period.",
    calc(
      `
base_value = 240
current_value = 318
index_level = current_value / base_value * 100
index_change = index_level - 100
`,
      "Index level: {{=index_level}}\nIndex change: {{=index_change}}\n",
    ),
    ["index", "ratio"],
  ),
  template(
    "chart-business-kpi",
    "Charts",
    "chart",
    "KPI bar chart",
    "Creates a simple bar chart for business KPIs.",
    fenced(
      "chart",
      `
type: bar
title: Quarterly KPI plan
data:
  - metric: Revenue
    value: 125
  - metric: Margin
    value: 61
  - metric: NPS
    value: 48
x: metric
y: value
`,
    ),
    ["chart", "kpi"],
  ),
  template(
    "chart-finance-line",
    "Charts",
    "chart",
    "Revenue trend line",
    "Creates a line chart for monthly revenue.",
    fenced(
      "chart",
      `
type: line
title: Monthly revenue
data:
  - month: Jan
    revenue: 420
  - month: Feb
    revenue: 460
  - month: Mar
    revenue: 515
x: month
y: revenue
`,
    ),
    ["chart", "revenue"],
  ),
  template(
    "chart-business-horizontal-risk",
    "Charts",
    "chart",
    "Horizontal risk comparison",
    "Creates a horizontal bar chart for ranked business comparisons with long labels.",
    fenced(
      "chart",
      `
type: horizontal-bar
title: Renewal risk by account
target: 40
targetLabel: Escalation
valueSuffix: "%"
data:
  - account: Very Long Enterprise Account
    risk: 72
  - account: Growth Segment
    risk: 34
  - account: Expansion Candidate
    risk: 18
x: account
y: risk
`,
    ),
    ["chart", "risk", "horizontal"],
  ),
  template(
    "vega-lite-grouped",
    "Charts",
    "vega-lite",
    "Grouped Vega-Lite bar",
    "Creates a grouped static Vega-Lite bar chart.",
    fenced(
      "vega-lite",
      `{
  "mark": "bar",
  "data": {
    "values": [
      { "region": "East", "quarter": "Q1", "revenue": 120 },
      { "region": "East", "quarter": "Q2", "revenue": 145 },
      { "region": "West", "quarter": "Q1", "revenue": 98 },
      { "region": "West", "quarter": "Q2", "revenue": 132 }
    ]
  },
  "encoding": {
    "x": { "field": "quarter" },
    "y": { "field": "revenue" },
    "color": { "field": "region" }
  }
}`,
    ),
    ["vega-lite", "grouped"],
  ),
  template(
    "vega-lite-risk-ticks",
    "Charts",
    "vega-lite",
    "Risk score tick plot",
    "Creates a compact Vega-Lite tick plot for score distributions and QA review lanes.",
    fenced(
      "vega-lite",
      `{
  "mark": { "type": "tick" },
  "title": "Risk score distribution",
  "data": {
    "values": [
      { "team": "Legal", "risk": 72, "lane": "Review" },
      { "team": "Finance", "risk": 58, "lane": "Review" },
      { "team": "Operations", "risk": 41, "lane": "Ready" }
    ]
  },
  "encoding": {
    "x": { "field": "team", "title": "Team" },
    "y": { "field": "risk", "title": "Risk score" },
    "color": { "field": "lane" }
  }
}`,
    ),
    ["vega-lite", "tick", "risk", "qa"],
  ),
  template(
    "vega-lite-opportunity-scatter",
    "Charts",
    "vega-lite",
    "Opportunity scatter plot",
    "Creates a Vega-Lite circle scatter plot for opportunity, account, or portfolio scoring.",
    fenced(
      "vega-lite",
      `{
  "mark": { "type": "circle" },
  "title": "Opportunity score scatter",
  "data": {
    "values": [
      { "account": "Acme", "score": 82, "dealSize": 40, "segment": "Enterprise" },
      { "account": "Beta", "score": 55, "dealSize": 10, "segment": "Growth" },
      { "account": "Cobalt", "score": 68, "dealSize": 24, "segment": "Midmarket" }
    ]
  },
  "encoding": {
    "x": { "field": "account", "title": "Account" },
    "y": { "field": "score", "title": "Opportunity score" },
    "size": { "field": "dealSize" },
    "color": { "field": "segment" }
  }
}`,
    ),
    ["vega-lite", "circle", "scatter", "score"],
  ),
  template(
    "vega-lite-readiness-labels",
    "Charts",
    "vega-lite",
    "Readiness label plot",
    "Creates a labeled Vega-Lite score plot for milestone, QA, or release-readiness reviews.",
    fenced(
      "vega-lite",
      `{
  "mark": "text",
  "title": "Milestone readiness labels",
  "data": {
    "values": [
      { "stage": "Security", "score": 92, "label": "Ready", "lane": "Release" },
      { "stage": "Accessibility", "score": 74, "label": "Review", "lane": "Release" },
      { "stage": "Evidence", "score": 58, "label": "Blocked", "lane": "Proof" }
    ]
  },
  "encoding": {
    "x": { "field": "stage", "title": "Milestone" },
    "y": { "field": "score", "title": "Readiness" },
    "text": { "field": "label" },
    "color": { "field": "lane" }
  }
}`,
    ),
    ["vega-lite", "text", "readiness", "qa"],
  ),
  template(
    "vega-lite-sla-thresholds",
    "Charts",
    "vega-lite",
    "SLA threshold rules",
    "Creates Vega-Lite rule marks for target, warning, or compliance threshold lines.",
    fenced(
      "vega-lite",
      `{
  "mark": "rule",
  "title": "SLA threshold rules",
  "data": {
    "values": [
      { "threshold": 95, "lane": "Target", "label": "Committed SLA" },
      { "threshold": 80, "lane": "Watch", "label": "Review floor" }
    ]
  },
  "encoding": {
    "y": { "field": "threshold", "type": "quantitative", "title": "Score" },
    "color": { "field": "lane" },
    "text": { "field": "label" }
  }
}`,
    ),
    ["vega-lite", "rule", "threshold", "sla", "qa"],
  ),
  template(
    "timeline-launch",
    "Business",
    "timeline",
    "Launch timeline",
    "Creates a dated launch timeline.",
    fenced(
      "timeline",
      `
2026-06-01: Discovery complete
2026-06-15: Pilot launch
2026-07-01: General availability
2026-07-15: Executive readout
`,
    ),
    ["timeline", "launch"],
  ),
  template(
    "roadmap-quarter",
    "Business",
    "roadmap",
    "Quarterly roadmap",
    "Creates roadmap stages with status and owners.",
    fenced(
      "roadmap",
      `
Now: Harden editor ergonomics | status=active | owner=Docs
Next: Expand export proof | status=planned | owner=Platform
Later: Automate native QA | status=planned | owner=Desktop
`,
    ),
    ["roadmap", "planning"],
  ),
  template(
    "adr-decision",
    "Business",
    "adr",
    "Architecture decision",
    "Creates an ADR transform block with standard fields.",
    fenced(
      "adr",
      `
Status: Proposed
Context: State the pressure, constraint, or tradeoff.
Decision: State the decision in one sentence.
Consequences: List expected benefits, costs, and follow-up checks.
`,
    ),
    ["adr", "decision"],
  ),
  template(
    "mermaid-flow",
    "Diagrams",
    "mermaid",
    "Mermaid process flow",
    "Creates a process flow diagram.",
    fenced(
      "mermaid",
      `
flowchart LR
  Draft --> Review
  Review --> Approve
  Approve --> Publish
`,
    ),
    ["diagram", "workflow"],
  ),
  template(
    "pikchr-business-flow",
    "Diagrams",
    "pikchr",
    "Pikchr business flow",
    "Creates a compact business-flow diagram that works with the native fallback.",
    fenced("pikchr", `box "Intake"; arrow "score"; diamond "Gate"; arrow "approve"; box "Launch"`),
    ["diagram", "fallback"],
  ),
  template(
    "dot-dependency",
    "Diagrams",
    "dot",
    "DOT dependency graph",
    "Creates a directed dependency graph.",
    fenced(
      "dot",
      `
digraph G {
  rankdir=LR;
  Requirements -> Design -> Build -> Verify;
  Verify -> Release;
}
`,
    ),
    ["diagram", "graphviz"],
  ),
  template(
    "plantuml-sequence",
    "Diagrams",
    "plantuml",
    "PlantUML sequence",
    "Creates a sequence diagram for external PlantUML rendering.",
    fenced(
      "plantuml",
      `
@startuml
User -> NEditor: Insert template
NEditor -> Compiler: Run transform
Compiler --> NEditor: Static artifact
@enduml
`,
    ),
    ["diagram", "sequence"],
  ),
  template(
    "csv-budget",
    "Data",
    "csv",
    "CSV budget table",
    "Creates a CSV table with formula totals.",
    fenced(
      "csv",
      `
Item,Q1,Q2,Total
Implementation,12000,18000,=12000+18000
Training,3000,4000,=3000+4000
Support,2000,2500,=2000+2500
`,
      `caption="Quarterly rollout budget"`,
    ),
    ["csv", "budget"],
  ),
  template(
    "sql-database-query",
    "Data",
    "sql",
    "SQL database query",
    "Queries a trusted local SQLite database and renders the result as a document table.",
    fenced(
      "sql",
      `
SELECT
  name,
  amount
FROM results
ORDER BY amount DESC
LIMIT 25;
`,
      `database="data/example.sqlite"`,
    ),
    ["sql", "database", "sqlite", "table"],
  ),
  template(
    "json-schema-object",
    "Data",
    "json-schema",
    "JSON Schema object",
    "Creates a schema documentation transform.",
    fenced(
      "json-schema",
      `{
  "title": "Customer",
  "type": "object",
  "required": ["id", "email"],
  "properties": {
    "id": { "type": "string" },
    "email": { "type": "string", "format": "email" },
    "tier": { "type": "string", "enum": ["starter", "growth", "enterprise"] }
  }
}`,
    ),
    ["schema", "api"],
  ),
  template(
    "openapi-endpoint",
    "Data",
    "openapi",
    "OpenAPI endpoint",
    "Creates an API endpoint documentation transform.",
    fenced(
      "openapi",
      `
openapi: 3.1.0
info:
  title: Template API
  version: 1.0.0
paths:
  /reports:
    get:
      summary: List reports
      responses:
        "200":
          description: Report list
`,
    ),
    ["openapi", "api"],
  ),
  template(
    "qr-release",
    "Business",
    "qr",
    "Release QR code",
    "Creates a QR transform for a release URL or artifact path.",
    fenced("qr", `https://example.com/releases/neditor-report`),
    ["qr", "release"],
  ),
];

export const transformTemplateCategories = [...new Set(builtinTransformTemplates.map((template) => template.category))].sort();
export const transformTemplateKinds = [...new Set(builtinTransformTemplates.map((template) => template.transform))].sort();

export function transformTemplateMarkdown(template: Pick<TransformTemplate, "body">) {
  return `${template.body.trimEnd()}\n`;
}

export function transformTemplateFillFields(template: Pick<TransformTemplate, "body" | "transform">): TransformTemplateFillField[] {
  const fields: TransformTemplateFillField[] = [];
  const seen = new Set<string>();
  const fencePattern = /^(~~~|```)\s*([A-Za-z0-9_-]+)[^\n]*\n([\s\S]*?)\n\1[ \t]*$/gm;
  let match: RegExpExecArray | null;
  while ((match = fencePattern.exec(template.body))) {
    const transform = (match[2] || template.transform).toLowerCase();
    const content = match[3] || "";
    if (transform === "calc") {
      collectCalcFillFields(content, fields, seen);
    } else {
      collectStructuredFillFields(content, fields, seen);
    }
  }
  return fields.slice(0, 12);
}

export function buildTransformTemplateAssistance(input: TransformTemplateAssistanceInput): TransformTemplateAssistance[] {
  const templates = input.templates;
  const filtered = input.filteredTemplates.length ? input.filteredTemplates : templates;
  const query = (input.query || "").trim();
  const category = (input.category || "all").trim() || "all";
  const transform = (input.transform || "all").trim() || "all";
  const documentText = input.documentText || "";
  const selected = recommendedTemplate(filtered, query, documentText);
  const selectedFields = selected ? transformTemplateFillFields(selected) : [];
  const customFields = input.customTemplate ? transformTemplateFillFields({
    body: input.customTemplate.body || "",
    transform: input.customTemplate.transform || "calc",
  }) : [];
  const calcCount = templates.filter((template) => template.transform === "calc").length;
  const customCount = templates.filter((template) => template.source === "custom").length;
  const noteWords = wordCount(input.assistanceNotes || "");
  const baseSignals = [
    `Library templates: ${templates.length}`,
    `Filtered templates: ${filtered.length}`,
    `Calc templates: ${calcCount}`,
    `Custom templates: ${customCount}`,
    `Search: ${query || "none"}`,
    `Category: ${category}`,
    `Transform: ${transform}`,
  ];
  const selectedLabel = selected ? `${selected.name} (${selected.category} ${selected.transform})` : "no matching template";

  return [
    {
      stepId: "choose-template",
      stepLabel: "Choose the right template",
      suggestedAnswer: selected
        ? `Start with "${selected.name}" because it best matches the current filters and document context. Keep the filter narrow enough for business users: ${category === "all" ? "choose a category" : `category ${category}`}, ${transform === "all" ? "choose a transform kind" : `transform ${transform}`}, then insert only after the template purpose matches the narrative claim.`
        : "Broaden the search, category, or transform filters until at least one template matches the business question, calculation, chart, diagram, data, API, or publishing task.",
      rationale: "Template choice should be driven by the document's decision or evidence need, not by browsing a long library at random.",
      contextSignals: [...baseSignals, `Recommended template: ${selectedLabel}`],
      actionLabel: "Add template choice guidance",
    },
    {
      stepId: "fill-values",
      stepLabel: "Fill values responsibly",
      suggestedAnswer: selectedFields.length
        ? `Replace sample values for ${selectedFields.slice(0, 6).map((field) => field.name).join(", ")} with sourced document values, and keep assumptions visible when a value is estimated or pending review.`
        : "Identify the user-editable values in the transform body, replace examples with sourced inputs, and record any assumptions before the transform result is used in prose.",
      rationale: "Business and scientific transforms are only as reliable as their inputs; visible assumptions make downstream review possible.",
      contextSignals: [
        `Recommended template: ${selectedLabel}`,
        `Detected fill fields: ${selectedFields.length ? selectedFields.map((field) => field.name).join(", ") : "none"}`,
        `Custom draft fill fields: ${customFields.length ? customFields.map((field) => field.name).join(", ") : "none"}`,
      ],
      actionLabel: "Add fill-value guidance",
    },
    {
      stepId: "preview-and-verify",
      stepLabel: "Preview and verify",
      suggestedAnswer: selected
        ? `After inserting "${selected.name}", run the transform preview and verify that the rendered result, labels, captions, units, source values, and dependent narrative all agree before export.`
        : "After inserting any transform, run preview, inspect diagnostics, and verify rendered output before relying on the result in the document.",
      rationale: "Preview catches syntax, engine, unit, and rendering issues before a calculated or visual claim reaches review.",
      contextSignals: [
        `Document words: ${wordCount(documentText)}`,
        `Template output kind: ${selected?.transform || transform}`,
        `Assistance notes words: ${noteWords}`,
      ],
      actionLabel: "Add preview guidance",
    },
    {
      stepId: "review-handoff",
      stepLabel: "Prepare review handoff",
      suggestedAnswer: selected
        ? `Document why "${selected.name}" was used, which values were replaced, who owns verification, and which section depends on the result. Add the transform to the QA handoff when it affects a financial, scientific, compliance, or client-facing claim.`
        : "Create a transform handoff note that names the calculation or visual, source data, owner, verification status, and affected sections before distribution.",
      rationale: "Transform results influence claims and decisions, so reviewers need ownership, evidence, and dependency context rather than a silent rendered block.",
      contextSignals: [...baseSignals, `Assistance notes words: ${noteWords}`],
      actionLabel: "Add handoff guidance",
    },
  ];
}

export function createCustomTransformTemplateId() {
  return `custom-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}

export function blankCustomTransformTemplate(): CustomTransformTemplate {
  return {
    id: createCustomTransformTemplateId(),
    name: "Custom calculation",
    category: "Custom",
    transform: "calc",
    summary: "Reusable transform template.",
    body: calc("input = 1\noutput = input"),
    tags: ["custom"],
  };
}

export function normalizeCustomTransformTemplates(value: unknown): CustomTransformTemplate[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  const templates: CustomTransformTemplate[] = [];
  for (const item of value) {
    if (!item || typeof item !== "object") continue;
    const record = item as Record<string, unknown>;
    const id = stringValue(record.id) || createCustomTransformTemplateId();
    const body = stringValue(record.body);
    if (!body || seen.has(id)) continue;
    seen.add(id);
    templates.push({
      id,
      name: stringValue(record.name) || "Custom transform",
      category: stringValue(record.category) || "Custom",
      transform: stringValue(record.transform) || "calc",
      summary: stringValue(record.summary) || "Reusable transform template.",
      body,
      tags: stringArray(record.tags).slice(0, 12),
    });
  }
  return templates.slice(0, 100);
}

export interface SaveCustomTransformTemplateStateResult {
  templates: CustomTransformTemplate[];
  template: CustomTransformTemplate | null;
  changed: boolean;
}

export interface DeleteCustomTransformTemplateStateResult {
  templates: CustomTransformTemplate[];
  changed: boolean;
}

export function saveCustomTransformTemplateState(
  templates: CustomTransformTemplate[],
  template: CustomTransformTemplate,
): SaveCustomTransformTemplateStateResult {
  const normalizedTemplates = normalizeCustomTransformTemplates(templates);
  const [normalized] = normalizeCustomTransformTemplates([template]);
  if (!normalized) {
    return { templates: normalizedTemplates, template: null, changed: false };
  }
  const existingIndex = normalizedTemplates.findIndex((candidate) => candidate.id === normalized.id);
  if (existingIndex >= 0) {
    return {
      templates: normalizedTemplates.map((candidate, index) => (index === existingIndex ? normalized : candidate)),
      template: normalized,
      changed: true,
    };
  }
  return {
    templates: [...normalizedTemplates, normalized],
    template: normalized,
    changed: true,
  };
}

export function deleteCustomTransformTemplateState(
  templates: CustomTransformTemplate[],
  id: string,
): DeleteCustomTransformTemplateStateResult {
  const normalizedTemplates = normalizeCustomTransformTemplates(templates);
  const nextTemplates = normalizedTemplates.filter((template) => template.id !== id);
  return {
    templates: nextTemplates,
    changed: nextTemplates.length !== normalizedTemplates.length,
  };
}

function stringValue(value: unknown) {
  return typeof value === "string" ? value.trim() : "";
}

function stringArray(value: unknown) {
  if (!Array.isArray(value)) return [];
  return value.filter((item): item is string => typeof item === "string").map((item) => item.trim()).filter(Boolean);
}

function collectCalcFillFields(content: string, fields: TransformTemplateFillField[], seen: Set<string>) {
  for (const line of content.split("\n")) {
    const match = line.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+?)\s*$/);
    if (!match) continue;
    const name = match[1];
    const value = match[2].replace(/\s+#.*$/, "").trim();
    if (!isEditableLiteral(value) || seen.has(name)) continue;
    seen.add(name);
    fields.push({ name, value, source: "calc-assignment" });
  }
}

function collectStructuredFillFields(content: string, fields: TransformTemplateFillField[], seen: Set<string>) {
  for (const line of content.split("\n")) {
    const match = line.match(/^([A-Za-z][A-Za-z0-9 _-]{1,40}):\s*(.+?)\s*$/);
    if (!match) continue;
    const name = match[1].trim();
    const value = match[2].trim();
    if (!value || value === "|" || value.startsWith("{") || value.startsWith("[") || seen.has(name)) continue;
    seen.add(name);
    fields.push({ name, value, source: "structured-field" });
  }
}

function isEditableLiteral(value: string) {
  return (
    /^[+-]?\d+(?:\.\d+)?(?:e[+-]?\d+)?$/i.test(value) ||
    /^(['"]).*\1$/.test(value) ||
    /^(true|false)$/i.test(value)
  );
}

function recommendedTemplate(templates: TransformTemplate[], query: string, documentText: string) {
  if (!templates.length) return null;
  const terms = new Set(
    `${query} ${documentText}`
      .toLowerCase()
      .split(/[^a-z0-9]+/)
      .filter((term) => term.length > 2),
  );
  let best = templates[0];
  let bestScore = -1;
  for (const template of templates) {
    const haystack = [template.name, template.category, template.transform, template.summary, ...template.tags].join(" ").toLowerCase();
    const score = [...terms].reduce((total, term) => total + (haystack.includes(term) ? 1 : 0), 0) + (template.transform === "calc" ? 0.25 : 0);
    if (score > bestScore) {
      best = template;
      bestScore = score;
    }
  }
  return best;
}

function wordCount(value: string) {
  return value.trim() ? value.trim().split(/\s+/).length : 0;
}
