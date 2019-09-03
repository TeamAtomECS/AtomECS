output = read_output('pos.txt');
pos = {output.vec};
%%
fh = gcf;
set(gcf, 'Color', 'w');
iter = 1;
while ishandle(fh) && iter <= length(pos)
    p = pos{iter};
    plot3(p(:,1), p(:,2), p(:,3), '.');
    
    iter = iter+1;
    
    axis equal;
  
    view(90,0);
    xlim([-0.5 0.5]);
    ylim([-0.5 0.5]);
    zlim([-0.5 0.5]);
    pause(0.01);
end

%%
if ishandle(fh)
    clf;
p = pos{6000};
plot3(p(1,:), p(2,:), p(3,:), '.');
end
