use crate::npm_registry::report_of;
use crate::python_fixtures::summaries;
use crate::ruby_fixtures::{Change, diff};

#[test]
fn own_code_that_reaches_the_network_through_a_path_gem_is_a_note() {
    let run = diff(Change {
        base: &[
            (
                "Gemfile",
                "source \"https://rubygems.org\"\ngem \"billing\", path: \"engines/billing\"\n",
            ),
            (
                "Gemfile.lock",
                "PATH\n  remote: engines/billing\n  specs:\n    billing (0.1.0)\n\nPLATFORMS\n  ruby\n\nDEPENDENCIES\n  billing!\n",
            ),
            (
                "engines/billing/billing.gemspec",
                "Gem::Specification.new do |spec|\n  spec.name = \"billing\"\nend\n",
            ),
            (
                "engines/billing/lib/billing.rb",
                "require \"billing/charger\"\n",
            ),
            (
                "engines/billing/lib/billing/charger.rb",
                "module Billing\n  class Charger\n    def self.charge\n      Net::HTTP.get(URI(\"https://example.com\"))\n    end\n  end\nend\n",
            ),
        ],
        head: &[(
            "app.rb",
            "require \"billing\"\n\nclass App\n  def main\n    Billing::Charger.charge\n  end\nend\n",
        )],
    });

    assert_eq!(
        summaries(&report_of(&run)["notes"]),
        vec![
            ". network engines/billing app.rb:5 app.App.main > engines/billing/lib/billing/charger.Charger.charge > Net::HTTP"
        ],
        "{}",
        run.stderr
    );
}

#[test]
fn requiring_a_file_runs_its_top_level_code() {
    let run = diff(Change {
        base: &[
            ("Gemfile", "source \"https://rubygems.org\"\n"),
            ("Gemfile.lock", "PLATFORMS\n  ruby\n"),
            ("engines/tools/tools.gemspec", "Gem::Specification.new\n"),
            ("engines/tools/lib/boot.rb", "system(\"make\")\n"),
        ],
        head: &[(
            "app.rb",
            "require \"boot\"\n\nclass App\n  def main\n  end\nend\n",
        )],
    });

    assert_eq!(
        summaries(&report_of(&run)["notes"]),
        vec![". exec engines/tools app.rb:1 app.init > engines/tools/lib/boot.init > system"],
        "{}",
        run.stderr
    );
}

#[test]
fn a_gem_whose_path_is_the_repository_itself_is_own_code() {
    let run = diff(Change {
        base: &[
            ("Gemfile", "source \"https://rubygems.org\"\ngemspec\n"),
            (
                "Gemfile.lock",
                "PATH\n  remote: .\n  specs:\n    packer (1.0.0)\n\nPLATFORMS\n  ruby\n\nDEPENDENCIES\n  packer!\n",
            ),
            ("packer.gemspec", "Gem::Specification.new\n"),
            ("lib/packer.rb", "module Packer\nend\n"),
        ],
        head: &[(
            "app.rb",
            "require \"packer/generated\"\n\nclass App\n  def main\n  end\nend\n",
        )],
    });

    assert_eq!(
        report_of(&run)["skipped"],
        serde_json::Value::Array(Vec::new()),
        "{}",
        run.stderr
    );
}
