{
  description = "My flake templates";

  outputs = { self }: {
    templates = {
      rust = {
        path = ./rust;
        description = "rust flake using naersk";
      };
    };
  };
}
