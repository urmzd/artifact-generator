from importlib.resources import files


def load_dashboard() -> str:
    return files("artifact_generator.assets").joinpath("dashboard.html").read_text()
