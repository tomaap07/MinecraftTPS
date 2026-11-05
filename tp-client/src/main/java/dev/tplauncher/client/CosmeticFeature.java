package dev.tplauncher.client;

import net.minecraft.client.model.*;
import net.minecraft.client.network.AbstractClientPlayerEntity;
import net.minecraft.client.render.*;
import net.minecraft.client.render.entity.PlayerEntityRenderer;
import net.minecraft.client.render.entity.feature.FeatureRenderer;
import net.minecraft.client.render.entity.model.PlayerEntityModel;
import net.minecraft.client.util.math.MatrixStack;
import net.minecraft.util.Identifier;

public final class CosmeticFeature extends FeatureRenderer<AbstractClientPlayerEntity,PlayerEntityModel<AbstractClientPlayerEntity>> {
    private static final Identifier TEXTURE=Identifier.of("tp_client","textures/celestial.png");
    private final ModelPart left,right,crown;
    public CosmeticFeature(PlayerEntityRenderer context){
        super(context);ModelData data=new ModelData();var root=data.getRoot();
        root.addChild("left",ModelPartBuilder.create().uv(0,0).cuboid(0,-1,0,12,2,1).uv(0,4).cuboid(1,1,0,10,3,1).uv(0,8).cuboid(2,4,0,8,3,1).uv(0,12).cuboid(3,7,0,5,3,1),ModelTransform.pivot(3,2,3));
        root.addChild("right",ModelPartBuilder.create().uv(0,0).mirrored().cuboid(-12,-1,0,12,2,1).uv(0,4).cuboid(-11,1,0,10,3,1).uv(0,8).cuboid(-10,4,0,8,3,1).uv(0,12).cuboid(-8,7,0,5,3,1),ModelTransform.pivot(-3,2,3));
        root.addChild("crown",ModelPartBuilder.create().uv(32,0).cuboid(-5,-12,-5,10,1,1).cuboid(-5,-12,4,10,1,1).cuboid(-5,-12,-4,1,1,8).cuboid(4,-12,-4,1,1,8).uv(32,12).cuboid(-1,-15,-5,2,3,1).cuboid(-1,-15,4,2,3,1).cuboid(-5,-15,-1,1,3,2).cuboid(4,-15,-1,1,3,2),ModelTransform.NONE);
        ModelPart model=TexturedModelData.of(data,64,32).createModel();left=model.getChild("left");right=model.getChild("right");crown=model.getChild("crown");
    }
    @Override public void render(MatrixStack matrices,VertexConsumerProvider consumers,int light,AbstractClientPlayerEntity player,float limbAngle,float limbDistance,float tickDelta,float progress,float headYaw,float headPitch){
        if(!Cosmetics.local(player))return;
        var buffer=consumers.getBuffer(RenderLayer.getEntityCutoutNoCull(TEXTURE));
        if(Cosmetics.wings()){
            matrices.push();getContextModel().body.rotate(matrices);float flap=(float)Math.sin((player.age+tickDelta)*0.045)*0.12f;
            left.yaw=-0.5f-flap;right.yaw=0.5f+flap;left.roll=-0.12f;right.roll=0.12f;
            left.render(matrices,buffer,light,OverlayTexture.DEFAULT_UV);right.render(matrices,buffer,light,OverlayTexture.DEFAULT_UV);matrices.pop();
        }
        if(Cosmetics.crown()){matrices.push();getContextModel().head.rotate(matrices);matrices.translate(0,Math.sin((player.age+tickDelta)*0.05)*0.015,0);crown.render(matrices,buffer,light,OverlayTexture.DEFAULT_UV);matrices.pop();}
    }
}
