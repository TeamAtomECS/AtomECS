function output = read_collisions()
    file = "collisions.txt";
    fh = fopen(file, 'r');
    oc = onCleanup(@() fclose(fh));

    output = {};
    while ~feof(fh)
        % example header: step 0, 1
        header_data = sscanf(fgetl(fh), '%d');
        step = header_data(1);
        collisions = sscanf(fgetl(fh), '%f ');
        atoms = sscanf(fgetl(fh), '%f ');
        particles = sscanf(fgetl(fh), '%f ');
        output{end+1} = struct('collisions', collisions, 'atoms', atoms, 'particles', particles);
    end
    output = cat(1, output{:});
end


